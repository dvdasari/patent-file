-- FER (First Examination Report) Response Assistant tables

-- Main FER analysis record
CREATE TABLE fer_analyses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    project_id UUID REFERENCES projects(id) ON DELETE SET NULL,
    title TEXT NOT NULL DEFAULT 'Untitled FER Analysis',
    fer_text TEXT NOT NULL,
    application_number TEXT,
    fer_date DATE,
    examiner_name TEXT,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'parsing', 'parsed', 'generating', 'complete', 'failed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_fer_analyses_user ON fer_analyses(user_id);

-- Parsed objections from FER
CREATE TABLE fer_objections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    analysis_id UUID NOT NULL REFERENCES fer_analyses(id) ON DELETE CASCADE,
    objection_number INT NOT NULL,
    category TEXT NOT NULL
        CHECK (category IN ('novelty', 'inventive_step', 'non_patentable', 'insufficiency', 'unity', 'formal', 'other')),
    section_reference TEXT,
    summary TEXT NOT NULL,
    full_text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_fer_objections_analysis ON fer_objections(analysis_id);

-- AI-generated responses to each objection
CREATE TABLE fer_responses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    objection_id UUID NOT NULL REFERENCES fer_objections(id) ON DELETE CASCADE,
    legal_arguments TEXT NOT NULL DEFAULT '',
    claim_amendments TEXT NOT NULL DEFAULT '',
    case_law_citations TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'generating', 'complete', 'edited', 'accepted')),
    user_edited_text TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_fer_responses_objection ON fer_responses(objection_id);

-- Trigger for updated_at
CREATE TRIGGER set_fer_analyses_updated_at
    BEFORE UPDATE ON fer_analyses
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER set_fer_responses_updated_at
    BEFORE UPDATE ON fer_responses
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
