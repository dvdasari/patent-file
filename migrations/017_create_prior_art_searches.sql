-- Prior art search queries
CREATE TABLE prior_art_searches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    project_id UUID REFERENCES projects(id) ON DELETE SET NULL,
    query_text TEXT NOT NULL,
    ipc_classification TEXT,
    applicant_filter TEXT,
    date_from DATE,
    date_to DATE,
    include_npl BOOLEAN NOT NULL DEFAULT false,
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, searching, analyzing, complete, failed
    result_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_prior_art_searches_user_id ON prior_art_searches(user_id);

CREATE TRIGGER trg_prior_art_searches_updated_at BEFORE UPDATE ON prior_art_searches
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Individual search results
CREATE TABLE prior_art_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    search_id UUID NOT NULL REFERENCES prior_art_searches(id) ON DELETE CASCADE,
    source TEXT NOT NULL,             -- inpass, google_patents, csir, npl
    external_id TEXT,                 -- patent/application number
    title TEXT NOT NULL,
    applicant TEXT,
    filing_date DATE,
    publication_date DATE,
    ipc_codes TEXT,                   -- comma-separated
    abstract_text TEXT,
    url TEXT,
    similarity_score REAL NOT NULL DEFAULT 0.0,  -- 0.0 to 1.0
    novelty_assessment TEXT,          -- AI-generated plain-language assessment
    relevance_rank INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_prior_art_results_search_id ON prior_art_results(search_id);
CREATE INDEX idx_prior_art_results_similarity ON prior_art_results(search_id, similarity_score DESC);

-- Search report exports
CREATE TABLE search_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    search_id UUID NOT NULL REFERENCES prior_art_searches(id) ON DELETE CASCADE,
    format TEXT NOT NULL DEFAULT 'pdf',
    storage_path TEXT NOT NULL,
    file_size_bytes BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_search_reports_search_id ON search_reports(search_id);
