CREATE TABLE interview_responses (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    step_number     INT NOT NULL,
    question_key    TEXT NOT NULL,
    question_text   TEXT NOT NULL,
    response_text   TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(project_id, question_key)
);

CREATE INDEX idx_interview_responses_project_id ON interview_responses(project_id);

CREATE TRIGGER trg_interview_responses_updated_at BEFORE UPDATE ON interview_responses
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
