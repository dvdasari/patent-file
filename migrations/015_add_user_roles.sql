-- Add role-based access control to users.
-- Roles: 'inventor' (default), 'patent_agent', 'admin'

CREATE TYPE user_role AS ENUM ('inventor', 'patent_agent', 'admin');

ALTER TABLE users ADD COLUMN role user_role NOT NULL DEFAULT 'inventor';
