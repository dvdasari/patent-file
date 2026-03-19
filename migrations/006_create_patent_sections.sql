CREATE TABLE patent_sections (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    section_type    TEXT NOT NULL,
    content         TEXT NOT NULL,
    ai_generated    BOOLEAN NOT NULL DEFAULT true,
    edit_count      INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(project_id, section_type)
);

CREATE INDEX idx_patent_sections_project_id ON patent_sections(project_id);

CREATE TRIGGER trg_patent_sections_updated_at BEFORE UPDATE ON patent_sections
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
