CREATE TABLE subscriptions (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                     UUID NOT NULL REFERENCES users(id),
    razorpay_customer_id        TEXT NOT NULL,
    razorpay_subscription_id    TEXT NOT NULL UNIQUE,
    plan_id                     TEXT NOT NULL,
    status                      TEXT NOT NULL DEFAULT 'active',
    current_period_start        TIMESTAMPTZ NOT NULL,
    current_period_end          TIMESTAMPTZ NOT NULL,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_subscriptions_user_id ON subscriptions(user_id);
CREATE INDEX idx_subscriptions_razorpay_sub_id ON subscriptions(razorpay_subscription_id);

CREATE TRIGGER trg_subscriptions_updated_at BEFORE UPDATE ON subscriptions
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
