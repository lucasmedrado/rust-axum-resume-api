use std::error::Error;

use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use vitae_backend::config::Config;
use vitae_backend::routes::build_router;
use vitae_backend::state::AppState;

#[tokio::main]
async fn main() {
    // `run` devolve o erro com `Display` (legível); retornar o erro direto do `main`
    // imprimiria o formato `Debug`.
    if let Err(err) = run().await {
        eprintln!("erro fatal: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("vitae_backend=debug,tower_http=debug")),
        )
        .init();

    let config = Config::from_env()?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;
    tracing::info!("migrations aplicadas");

    let listener = TcpListener::bind(config.bind_addr).await?;
    tracing::info!("servidor rodando em http://{}", listener.local_addr()?); 

    let app = build_router(AppState::new(pool, config));

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Espera Ctrl+C para encerrar o servidor terminando as requisições em andamento.
async fn shutdown_signal() {
    if let Err(err) = tokio::signal::ctrl_c().await {
        tracing::error!("falha ao escutar Ctrl+C: {err}");
    }
    tracing::info!("encerrando servidor...");
}
