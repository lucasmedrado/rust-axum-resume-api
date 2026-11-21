//! Testes de integração: sobem o roteador completo e fazem requisições HTTP em memória
//! contra um Postgres real. `#[sqlx::test]` cria um banco novo (com as migrations) para cada teste.
//!
//! Requer a variável `DATABASE_URL` apontando para um Postgres cujo usuário possa criar bancos.

use axum::Router;
use axum::body::Body;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

use vitae_backend::config::Config;
use vitae_backend::routes::build_router;
use vitae_backend::state::AppState;

// ---------- helpers ----------

fn test_app(pool: PgPool) -> Router {
    let config = Config {
        database_url: String::new(),
        jwt_secret: "segredo-de-teste-com-mais-de-32-caracteres".into(),
        jwt_ttl_hours: 1,
        bind_addr: "127.0.0.1:0".parse().unwrap(),
    };
    build_router(AppState::new(pool, config))
}

async fn send(
    app: &Router,
    method: Method,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header(AUTHORIZATION, format!("Bearer {token}"));
    }
    let body = match body {
        Some(json) => {
            builder = builder.header(CONTENT_TYPE, "application/json");
            Body::from(json.to_string())
        }
        None => Body::empty(),
    };

    let response = app
        .clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .unwrap();

    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };

    (status, json)
}

/// Cadastra um usuário e devolve o token.
async fn register(app: &Router, email: &str) -> String {
    let (status, body) = send(
        app,
        Method::POST,
        "/users",
        None,
        Some(json!({ "email": email, "password": "senha-segura-123" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body["token"].as_str().unwrap().to_string()
}

async fn create_profile(app: &Router, token: &str, slug: &str) {
    let (status, body) = send(
        app,
        Method::POST,
        "/profiles/me",
        Some(token),
        Some(json!({
            "full_name": "Maria Silva",
            "localization": "São Paulo, SP",
            "slug": slug,
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
}

fn experience_body() -> Value {
    json!({
        "company_name": "Acme",
        "position": "Desenvolvedora Rust",
        "start_date": "2022-03-01",
    })
}

// ---------- usuários e autenticação ----------

#[sqlx::test]
async fn cadastro_login_e_consulta_do_proprio_usuario(pool: PgPool) {
    let app = test_app(pool);

    let token = register(&app, "Maria@Exemplo.com").await;

    // O token do cadastro já funciona; o email é normalizado e o hash nunca vaza.
    let (status, me) = send(&app, Method::GET, "/users/me", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["email"], "maria@exemplo.com");
    assert!(me.get("password_hash").is_none());

    // Login com a senha correta (ignorando maiúsculas no email).
    let (status, login) = send(
        &app,
        Method::POST,
        "/auth/login",
        None,
        Some(json!({ "email": "MARIA@exemplo.com", "password": "senha-segura-123" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(login["token"].as_str().is_some());
}

#[sqlx::test]
async fn login_com_senha_errada_ou_email_inexistente_retorna_401(pool: PgPool) {
    let app = test_app(pool);
    register(&app, "maria@exemplo.com").await;

    for (email, password) in [
        ("maria@exemplo.com", "senha-errada-123"),
        ("ninguem@exemplo.com", "senha-segura-123"),
    ] {
        let (status, body) = send(
            &app,
            Method::POST,
            "/auth/login",
            None,
            Some(json!({ "email": email, "password": password })),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
    }
}

#[sqlx::test]
async fn cadastro_com_email_repetido_retorna_409(pool: PgPool) {
    let app = test_app(pool);
    register(&app, "maria@exemplo.com").await;

    let (status, _) = send(
        &app,
        Method::POST,
        "/users",
        None,
        Some(json!({ "email": "MARIA@exemplo.com", "password": "senha-segura-123" })),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn cadastro_invalido_retorna_422_com_detalhes_por_campo(pool: PgPool) {
    let app = test_app(pool);

    let (status, body) = send(
        &app,
        Method::POST,
        "/users",
        None,
        Some(json!({ "email": "nao-e-email", "password": "curta" })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body["fields"]["email"].is_array());
    assert!(body["fields"]["password"].is_array());
}

#[sqlx::test]
async fn json_malformado_retorna_400(pool: PgPool) {
    let app = test_app(pool);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/users")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from("{ isto nao e json"))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn rotas_protegidas_exigem_token_valido(pool: PgPool) {
    let app = test_app(pool);

    let (sem_token, _) = send(&app, Method::GET, "/users/me", None, None).await;
    let (token_ruim, _) = send(&app, Method::GET, "/users/me", Some("lixo"), None).await;

    assert_eq!(sem_token, StatusCode::UNAUTHORIZED);
    assert_eq!(token_ruim, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn troca_de_senha_exige_senha_atual_e_vale_no_proximo_login(pool: PgPool) {
    let app = test_app(pool);
    let token = register(&app, "maria@exemplo.com").await;

    // Senha atual incorreta
    let (status, _) = send(
        &app,
        Method::PATCH,
        "/users/me",
        Some(&token),
        Some(json!({ "current_password": "errada-errada", "new_password": "nova-senha-456" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Senha atual correta
    let (status, _) = send(
        &app,
        Method::PATCH,
        "/users/me",
        Some(&token),
        Some(json!({ "current_password": "senha-segura-123", "new_password": "nova-senha-456" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let login = |password: &'static str| {
        send(
            &app,
            Method::POST,
            "/auth/login",
            None,
            Some(json!({ "email": "maria@exemplo.com", "password": password })),
        )
    };
    assert_eq!(login("senha-segura-123").await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(login("nova-senha-456").await.0, StatusCode::OK);
}

#[sqlx::test]
async fn excluir_usuario_remove_perfil_e_experiencias_em_cascata(pool: PgPool) {
    let app = test_app(pool.clone());
    let token = register(&app, "maria@exemplo.com").await;
    create_profile(&app, &token, "maria-silva").await;
    send(
        &app,
        Method::POST,
        "/experiences",
        Some(&token),
        Some(experience_body()),
    )
    .await;

    let (status, _) = send(&app, Method::DELETE, "/users/me", Some(&token), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    for table in ["users", "profiles", "experiences"] {
        let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0, "tabela {table} deveria estar vazia");
    }
}

// ---------- perfil ----------

#[sqlx::test]
async fn ciclo_completo_do_perfil(pool: PgPool) {
    let app = test_app(pool);
    let token = register(&app, "maria@exemplo.com").await;

    // Ainda não existe
    let (status, _) = send(&app, Method::GET, "/profiles/me", Some(&token), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    create_profile(&app, &token, "maria-silva").await;

    // Atualização parcial: só o que foi enviado muda
    let (status, updated) = send(
        &app,
        Method::PATCH,
        "/profiles/me",
        Some(&token),
        Some(json!({ "localization": "Ourinhos, SP", "is_public": true })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["localization"], "Ourinhos, SP");
    assert_eq!(updated["full_name"], "Maria Silva");
    assert_eq!(updated["is_public"], true);

    let (status, _) = send(&app, Method::DELETE, "/profiles/me", Some(&token), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = send(&app, Method::GET, "/profiles/me", Some(&token), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn perfil_duplicado_ou_slug_em_uso_retorna_409(pool: PgPool) {
    let app = test_app(pool);
    let maria = register(&app, "maria@exemplo.com").await;
    let joao = register(&app, "joao@exemplo.com").await;
    create_profile(&app, &maria, "maria-silva").await;

    let body = json!({ "full_name": "Maria Outra", "localization": "Rio", "slug": "outro-slug" });
    let (segundo_perfil, _) =
        send(&app, Method::POST, "/profiles/me", Some(&maria), Some(body)).await;
    assert_eq!(segundo_perfil, StatusCode::CONFLICT);

    let body = json!({ "full_name": "João Souza", "localization": "Rio", "slug": "maria-silva" });
    let (slug_repetido, _) =
        send(&app, Method::POST, "/profiles/me", Some(&joao), Some(body)).await;
    assert_eq!(slug_repetido, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn perfil_com_slug_invalido_retorna_422(pool: PgPool) {
    let app = test_app(pool);
    let token = register(&app, "maria@exemplo.com").await;

    let (status, body) = send(
        &app,
        Method::POST,
        "/profiles/me",
        Some(&token),
        Some(json!({ "full_name": "Maria Silva", "localization": "SP", "slug": "Slug Inválido" })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body["fields"]["slug"].is_array());
}

// ---------- experiências ----------

#[sqlx::test]
async fn ciclo_completo_de_experiencias(pool: PgPool) {
    let app = test_app(pool);
    let token = register(&app, "maria@exemplo.com").await;
    create_profile(&app, &token, "maria-silva").await;

    // Cria duas
    let (status, primeira) = send(
        &app,
        Method::POST,
        "/experiences",
        Some(&token),
        Some(experience_body()),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{primeira}");
    let (status, _) = send(
        &app,
        Method::POST,
        "/experiences",
        Some(&token),
        Some(json!({
            "company_name": "Beta", "position": "Estagiária",
            "start_date": "2020-01-01", "end_date": "2021-12-31"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Lista (mais recente primeiro)
    let (status, lista) = send(&app, Method::GET, "/experiences", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(lista.as_array().unwrap().len(), 2);
    assert_eq!(lista[0]["company_name"], "Acme");

    // Atualiza SÓ a primeira (a segunda não pode ser afetada)
    let id = primeira["id"].as_str().unwrap();
    let (status, atualizada) = send(
        &app,
        Method::PATCH,
        &format!("/experiences/{id}"),
        Some(&token),
        Some(json!({ "position": "Tech Lead" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(atualizada["position"], "Tech Lead");
    assert_eq!(atualizada["company_name"], "Acme");

    let (_, lista) = send(&app, Method::GET, "/experiences", Some(&token), None).await;
    assert_eq!(lista[1]["position"], "Estagiária");

    // Remove
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/experiences/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, lista) = send(&app, Method::GET, "/experiences", Some(&token), None).await;
    assert_eq!(lista.as_array().unwrap().len(), 1);
}

#[sqlx::test]
async fn experiencia_exige_que_o_perfil_exista(pool: PgPool) {
    let app = test_app(pool);
    let token = register(&app, "maria@exemplo.com").await;

    let (status, _) = send(
        &app,
        Method::POST,
        "/experiences",
        Some(&token),
        Some(experience_body()),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn usuario_nao_acessa_experiencia_de_outro(pool: PgPool) {
    let app = test_app(pool);
    let maria = register(&app, "maria@exemplo.com").await;
    let joao = register(&app, "joao@exemplo.com").await;
    create_profile(&app, &maria, "maria-silva").await;
    create_profile(&app, &joao, "joao-souza").await;

    let (_, experiencia) = send(
        &app,
        Method::POST,
        "/experiences",
        Some(&maria),
        Some(experience_body()),
    )
    .await;
    let uri = format!("/experiences/{}", experiencia["id"].as_str().unwrap());

    let (editar, _) = send(
        &app,
        Method::PATCH,
        &uri,
        Some(&joao),
        Some(json!({ "position": "Hackeado" })),
    )
    .await;
    let (remover, _) = send(&app, Method::DELETE, &uri, Some(&joao), None).await;
    let (lista_do_joao, lista) = send(&app, Method::GET, "/experiences", Some(&joao), None).await;

    assert_eq!(editar, StatusCode::NOT_FOUND);
    assert_eq!(remover, StatusCode::NOT_FOUND);
    assert_eq!(lista_do_joao, StatusCode::OK);
    assert!(lista.as_array().unwrap().is_empty());

    // E a experiência da Maria continua intacta
    let (_, lista) = send(&app, Method::GET, "/experiences", Some(&maria), None).await;
    assert_eq!(lista[0]["position"], "Desenvolvedora Rust");
}

#[sqlx::test]
async fn periodo_invalido_de_experiencia_retorna_422(pool: PgPool) {
    let app = test_app(pool);
    let token = register(&app, "maria@exemplo.com").await;
    create_profile(&app, &token, "maria-silva").await;

    // Validação do DTO (as duas datas na requisição)
    let (status, _) = send(
        &app,
        Method::POST,
        "/experiences",
        Some(&token),
        Some(json!({
            "company_name": "Acme", "position": "Dev",
            "start_date": "2022-01-01", "end_date": "2020-01-01"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Validação do banco (só end_date na requisição, contra start_date já salva)
    let (_, criada) = send(
        &app,
        Method::POST,
        "/experiences",
        Some(&token),
        Some(experience_body()),
    )
    .await;
    let (status, _) = send(
        &app,
        Method::PATCH,
        &format!("/experiences/{}", criada["id"].as_str().unwrap()),
        Some(&token),
        Some(json!({ "end_date": "2000-01-01" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

// ---------- diversos ----------

#[sqlx::test]
async fn health_e_rota_inexistente(pool: PgPool) {
    let app = test_app(pool);

    let (status, body) = send(&app, Method::GET, "/health", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");

    let (status, _) = send(&app, Method::GET, "/nao-existe", None, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ---------- formações acadêmicas ----------

#[sqlx::test]
async fn ciclo_completo_de_formacoes_academicas(pool: PgPool) {
    let app = test_app(pool);
    let token = register(&app, "maria@exemplo.com").await;
    create_profile(&app, &token, "maria-silva").await;

    let (status, criada) = send(
        &app,
        Method::POST,
        "/academic-educations",
        Some(&token),
        Some(json!({
            "institution": "Fatec Ourinhos",
            "degree": "Tecnólogo",
            "field_of_study": "Análise e Desenvolvimento de Sistemas",
            "start_date": "2014-08-01",
            "end_date": "2019-07-01"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{criada}");

    let (status, lista) = send(&app, Method::GET, "/academic-educations", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(lista.as_array().unwrap().len(), 1);
    assert_eq!(lista[0]["institution"], "Fatec Ourinhos");

    let id = criada["id"].as_str().unwrap();
    let (status, atualizada) = send(
        &app,
        Method::PATCH,
        &format!("/academic-educations/{id}"),
        Some(&token),
        Some(json!({ "degree": "Graduação Tecnológica" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(atualizada["degree"], "Graduação Tecnológica");
    assert_eq!(atualizada["institution"], "Fatec Ourinhos");

    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/academic-educations/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn formacao_academica_isolamento_e_validacao(pool: PgPool) {
    let app = test_app(pool);
    let maria = register(&app, "maria@exemplo.com").await;
    let joao = register(&app, "joao@exemplo.com").await;
    create_profile(&app, &maria, "maria-silva").await;
    create_profile(&app, &joao, "joao-souza").await;

    let (_, criada) = send(
        &app,
        Method::POST,
        "/academic-educations",
        Some(&maria),
        Some(json!({ "institution": "USP", "start_date": "2020-01-01" })),
    )
    .await;
    let uri = format!("/academic-educations/{}", criada["id"].as_str().unwrap());

    // João não consegue editar nem apagar formação de Maria
    assert_eq!(
        send(&app, Method::PATCH, &uri, Some(&joao), Some(json!({ "institution": "Hack" }))).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        send(&app, Method::DELETE, &uri, Some(&joao), None).await.0,
        StatusCode::NOT_FOUND
    );

    // Período inválido no banco (end_date < start_date salvo)
    assert_eq!(
        send(&app, Method::PATCH, &uri, Some(&maria), Some(json!({ "end_date": "2019-01-01" }))).await.0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

// ---------- certificações ----------

#[sqlx::test]
async fn ciclo_completo_de_certificacoes(pool: PgPool) {
    let app = test_app(pool);
    let token = register(&app, "maria@exemplo.com").await;
    create_profile(&app, &token, "maria-silva").await;

    let (status, criada) = send(
        &app,
        Method::POST,
        "/certifications",
        Some(&token),
        Some(json!({
            "name": "Rust Systems Programming",
            "issuing_organization": "Linux Foundation",
            "issue_date": "2024-05-10",
            "url": "https://example.com/cert/rust"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{criada}");

    let (status, lista) = send(&app, Method::GET, "/certifications", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(lista.as_array().unwrap().len(), 1);

    let id = criada["id"].as_str().unwrap();
    let (status, atualizada) = send(
        &app,
        Method::PATCH,
        &format!("/certifications/{id}"),
        Some(&token),
        Some(json!({ "issuing_organization": "CNCF" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(atualizada["issuing_organization"], "CNCF");

    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/certifications/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

// ---------- habilidades ----------

#[sqlx::test]
async fn ciclo_completo_de_habilidades_e_unicidade(pool: PgPool) {
    let app = test_app(pool);
    let token = register(&app, "maria@exemplo.com").await;
    create_profile(&app, &token, "maria-silva").await;

    let (status, criada) = send(
        &app,
        Method::POST,
        "/skills",
        Some(&token),
        Some(json!({
            "name": "Rust",
            "skill_type": "hard",
            "level": "advanced"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{criada}");

    // Duplicata case-insensitive no mesmo perfil deve retornar 409
    let (status, _) = send(
        &app,
        Method::POST,
        "/skills",
        Some(&token),
        Some(json!({
            "name": "rust",
            "skill_type": "hard",
            "level": "expert"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    let id = criada["id"].as_str().unwrap();
    let (status, atualizada) = send(
        &app,
        Method::PATCH,
        &format!("/skills/{id}"),
        Some(&token),
        Some(json!({ "level": "expert" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(atualizada["level"], "expert");

    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/skills/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

// ---------- links sociais ----------

#[sqlx::test]
async fn ciclo_completo_de_links_sociais_e_unicidade(pool: PgPool) {
    let app = test_app(pool);
    let token = register(&app, "maria@exemplo.com").await;
    create_profile(&app, &token, "maria-silva").await;

    let (status, criado) = send(
        &app,
        Method::POST,
        "/social-links",
        Some(&token),
        Some(json!({
            "platform_name": "GitHub",
            "url": "https://github.com/maria-silva"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{criado}");

    // Plataforma duplicada case-insensitive no mesmo perfil retorna 409
    let (status, _) = send(
        &app,
        Method::POST,
        "/social-links",
        Some(&token),
        Some(json!({
            "platform_name": "github",
            "url": "https://github.com/outra"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    let id = criado["id"].as_str().unwrap();
    let (status, atualizado) = send(
        &app,
        Method::PATCH,
        &format!("/social-links/{id}"),
        Some(&token),
        Some(json!({ "url": "https://github.com/maria-dev" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(atualizado["url"], "https://github.com/maria-dev");

    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/social-links/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}
