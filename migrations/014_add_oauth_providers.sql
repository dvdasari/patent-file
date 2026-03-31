-- Add OAuth provider support to users table.
-- password_hash becomes nullable for OAuth-only users.

ALTER TABLE users ADD COLUMN oauth_provider TEXT;
ALTER TABLE users ADD COLUMN oauth_provider_id TEXT;
ALTER TABLE users ALTER COLUMN password_hash DROP NOT NULL;

-- A user can only have one account per provider
CREATE UNIQUE INDEX idx_users_oauth_provider
    ON users (oauth_provider, oauth_provider_id)
    WHERE oauth_provider IS NOT NULL;
