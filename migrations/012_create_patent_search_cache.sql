-- Cache for prior art / patent search results
-- Avoids hammering the Lens.org API on repeated identical queries
CREATE TABLE patent_search_cache (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    query_hash     TEXT        NOT NULL,
    query_text     TEXT        NOT NULL,
    page           INT         NOT NULL DEFAULT 1,
    per_page       INT         NOT NULL DEFAULT 10,
    results        JSONB       NOT NULL,
    total          BIGINT      NOT NULL DEFAULT 0,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at     TIMESTAMPTZ NOT NULL DEFAULT now() + INTERVAL '24 hours'
);

-- Primary lookup: hash + pagination combo
CREATE INDEX idx_patent_search_cache_lookup
    ON patent_search_cache (query_hash, page, per_page);

-- TTL sweep: lets a background process (or the app itself) prune stale rows
CREATE INDEX idx_patent_search_cache_expires
    ON patent_search_cache (expires_at);
