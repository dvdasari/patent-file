CREATE TABLE section_versions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    section_id      UUID NOT NULL REFERENCES patent_sections(id) ON DELETE CASCADE,
    content         TEXT NOT NULL,
    version_number  INT NOT NULL,
    source          TEXT NOT NULL DEFAULT 'manual',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(section_id, version_number)
);

CREATE INDEX idx_section_versions_section_id ON section_versions(section_id);
