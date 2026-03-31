CREATE TABLE compliance_checks (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    run_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    total_warnings  INT NOT NULL DEFAULT 0,
    total_errors    INT NOT NULL DEFAULT 0,
    section10_passed BOOLEAN NOT NULL DEFAULT false,
    section3_passed  BOOLEAN NOT NULL DEFAULT false,
    claims_passed    BOOLEAN NOT NULL DEFAULT false,
    form2_compliant  BOOLEAN NOT NULL DEFAULT false,
    report_json     JSONB NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_compliance_checks_project ON compliance_checks(project_id);
CREATE INDEX idx_compliance_checks_run_at ON compliance_checks(project_id, run_at DESC);
