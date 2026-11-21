CREATE TABLE users (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    email         TEXT        NOT NULL UNIQUE,
    password_hash TEXT        NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT users_email_lowercase CHECK (email = lower(email)),
    CONSTRAINT users_email_not_blank CHECK (btrim(email) <> '')
);

-- Um usuário tem no máximo um perfil (UNIQUE em user_id).
CREATE TABLE profiles (
    id                     UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                UUID        NOT NULL UNIQUE REFERENCES users (id) ON DELETE CASCADE,
    full_name              TEXT        NOT NULL,
    localization           TEXT        NOT NULL,
    professional_objective TEXT,
    slug                   TEXT        NOT NULL UNIQUE,
    bio                    TEXT,
    is_public              BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT profiles_full_name_not_blank    CHECK (btrim(full_name) <> ''),
    CONSTRAINT profiles_localization_not_blank CHECK (btrim(localization) <> ''),
    -- minúsculas, números e hífen; sem hífen no início/fim nem hífens duplos
    CONSTRAINT profiles_slug_format CHECK (
        slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$' AND char_length(slug) BETWEEN 3 AND 60
    )
);

-- Listagem/consulta de perfis públicos.
CREATE INDEX profiles_public_idx ON profiles (slug) WHERE is_public;

CREATE TRIGGER users_set_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER profiles_set_updated_at
    BEFORE UPDATE ON profiles
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();