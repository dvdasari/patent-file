CREATE TABLE project_applicants (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id              UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    applicant_name          TEXT NOT NULL,
    applicant_address       TEXT NOT NULL,
    applicant_nationality   TEXT NOT NULL DEFAULT 'Indian',
    inventor_name           TEXT NOT NULL,
    inventor_address        TEXT NOT NULL,
    inventor_nationality    TEXT NOT NULL DEFAULT 'Indian',
    agent_name              TEXT,
    agent_registration_no   TEXT,
    assignee_name           TEXT,
    priority_date           DATE,
    priority_country        TEXT,
    priority_application_no TEXT,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(project_id)
);

CREATE INDEX idx_project_applicants_project_id ON project_applicants(project_id);

CREATE TRIGGER trg_project_applicants_updated_at BEFORE UPDATE ON project_applicants
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
