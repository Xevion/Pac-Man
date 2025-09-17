-- OAuth accounts linked to a single user
CREATE TABLE IF NOT EXISTS oauth_accounts (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  provider TEXT NOT NULL,
  provider_user_id TEXT NOT NULL,
  email TEXT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (provider, provider_user_id)
);

-- Ensure we can look up by email efficiently
CREATE INDEX IF NOT EXISTS idx_oauth_accounts_email ON oauth_accounts (email);

-- Optional: ensure users email uniqueness if desired; keep NULLs allowed
ALTER TABLE users
  ADD CONSTRAINT users_email_unique UNIQUE (email);
