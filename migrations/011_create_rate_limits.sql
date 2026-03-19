CREATE TABLE rate_limits (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id),
    action_type     TEXT NOT NULL,
    window_start    TIMESTAMPTZ NOT NULL,
    request_count   INT NOT NULL DEFAULT 1,
    UNIQUE(user_id, action_type, window_start)
);

CREATE INDEX idx_rate_limits_user_action ON rate_limits(user_id, action_type, window_start);
