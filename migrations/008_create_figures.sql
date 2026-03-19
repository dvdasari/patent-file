CREATE TABLE figures (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    sort_order      INT NOT NULL DEFAULT 0,
    description     TEXT NOT NULL,
    storage_path    TEXT NOT NULL,
    file_name       TEXT NOT NULL,
    content_type    TEXT NOT NULL,
    file_size_bytes BIGINT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_figures_project_id ON figures(project_id);
