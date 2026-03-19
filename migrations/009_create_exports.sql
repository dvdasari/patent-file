CREATE TABLE exports (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    format          TEXT NOT NULL,
    storage_path    TEXT NOT NULL,
    file_size_bytes BIGINT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_exports_project_id ON exports(project_id);
