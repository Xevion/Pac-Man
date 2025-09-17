-- Move provider-specific profile fields from users to oauth_accounts

-- Add provider profile fields to oauth_accounts
ALTER TABLE oauth_accounts
  ADD COLUMN IF NOT EXISTS username TEXT,
  ADD COLUMN IF NOT EXISTS display_name TEXT NULL,
  ADD COLUMN IF NOT EXISTS avatar_url TEXT NULL;

-- Drop provider-specific fields from users (keep email as canonical)
ALTER TABLE users
  DROP COLUMN IF EXISTS provider,
  DROP COLUMN IF EXISTS provider_user_id,
  DROP COLUMN IF EXISTS username,
  DROP COLUMN IF EXISTS display_name,
  DROP COLUMN IF EXISTS avatar_url;
