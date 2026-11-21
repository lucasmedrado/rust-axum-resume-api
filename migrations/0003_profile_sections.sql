-- Um perfil tem várias experiências.
CREATE TABLE experiences (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id   UUID        NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
    company_name TEXT        NOT NULL,
    position     TEXT        NOT NULL,
    start_date   DATE        NOT NULL,
    end_date     DATE,
    description  TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT experiences_period_check
        CHECK (end_date IS NULL OR end_date >= start_date),
    CONSTRAINT experiences_company_not_blank  CHECK (btrim(company_name) <> ''),
    CONSTRAINT experiences_position_not_blank CHECK (btrim(position) <> '')
);

CREATE TABLE academic_educations (
    id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id     UUID        NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
    institution    TEXT        NOT NULL,
    degree         TEXT,       -- Ex: Bacharelado, Mestrado
    field_of_study TEXT,       -- Ex: Engenharia de Software
    start_date     DATE,
    end_date       DATE,       -- NULL = em andamento
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT academic_educations_period_check
        CHECK (start_date IS NULL OR end_date IS NULL OR end_date >= start_date),
    CONSTRAINT academic_educations_institution_not_blank CHECK (btrim(institution) <> '')
);

CREATE TABLE certifications (
    id                   UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id           UUID        NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
    name                 TEXT        NOT NULL,
    issuing_organization TEXT,
    issue_date           DATE,
    url                  TEXT,       -- Link para o certificado
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT certifications_name_not_blank CHECK (btrim(name) <> ''),
    CONSTRAINT certifications_url_scheme CHECK (url IS NULL OR url ~* '^https?://')
);

CREATE TABLE skills (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID        NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
    name       TEXT        NOT NULL,
    skill_type TEXT        NOT NULL,
    level      TEXT,       -- Opcional
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT skills_name_not_blank CHECK (btrim(name) <> ''),
    CONSTRAINT skills_type_check  CHECK (skill_type IN ('hard', 'soft')),
    CONSTRAINT skills_level_check CHECK (
        level IS NULL OR level IN ('beginner', 'intermediate', 'advanced', 'expert')
    )
);

-- Impede a mesma skill duplicada no mesmo perfil (case-insensitive).
CREATE UNIQUE INDEX skills_profile_name_uidx ON skills (profile_id, lower(name));

CREATE TABLE social_links (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id    UUID        NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
    platform_name TEXT        NOT NULL, -- Ex: LinkedIn, GitHub, Twitter
    url           TEXT        NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT social_links_platform_not_blank CHECK (btrim(platform_name) <> ''),
    CONSTRAINT social_links_url_scheme CHECK (url ~* '^https?://')
);

CREATE UNIQUE INDEX social_links_profile_platform_uidx
    ON social_links (profile_id, lower(platform_name));

-- Postgres não cria índice automático para FKs: necessários para joins
-- por perfil e para o ON DELETE CASCADE não fazer seq scan.
CREATE INDEX experiences_profile_start_idx   ON experiences (profile_id, start_date DESC);
CREATE INDEX academic_educations_profile_idx ON academic_educations (profile_id);
CREATE INDEX certifications_profile_idx      ON certifications (profile_id);
-- skills e social_links já são cobertos pelos índices únicos compostos acima
-- (profile_id é a coluna líder).

CREATE TRIGGER experiences_set_updated_at
    BEFORE UPDATE ON experiences
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER academic_educations_set_updated_at
    BEFORE UPDATE ON academic_educations
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER certifications_set_updated_at
    BEFORE UPDATE ON certifications
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER skills_set_updated_at
    BEFORE UPDATE ON skills
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER social_links_set_updated_at
    BEFORE UPDATE ON social_links
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();