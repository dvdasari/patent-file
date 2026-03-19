use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use sqlx::PgPool;

use crate::error::AppError;

pub async fn razorpay_webhook(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    // TODO: Verify Razorpay webhook signature (HMAC-SHA256)
    // let _signature = headers.get("x-razorpay-signature");

    let event_type = body["event"]
        .as_str()
        .ok_or_else(|| AppError::bad_request("Missing event type"))?;

    tracing::info!("Razorpay webhook: {}", event_type);

    match event_type {
        "subscription.activated" | "subscription.charged" => {
            let payload = &body["payload"]["subscription"]["entity"];

            let sub_id = payload["id"].as_str().unwrap_or("");
            let customer_id = payload["customer_id"].as_str().unwrap_or("");
            let plan_id = payload["plan_id"].as_str().unwrap_or("");
            let status = if event_type == "subscription.activated" {
                "active"
            } else {
                "active"
            };

            // We'd need to map customer_id to user_id via a lookup
            // For now, log the event
            tracing::info!(
                "Subscription {}: sub_id={}, customer_id={}, plan_id={}",
                event_type, sub_id, customer_id, plan_id
            );
        }
        "subscription.cancelled" | "subscription.halted" => {
            let sub_id = body["payload"]["subscription"]["entity"]["id"]
                .as_str()
                .unwrap_or("");
            let new_status = if event_type == "subscription.cancelled" {
                "cancelled"
            } else {
                "halted"
            };

            sqlx::query(
                "UPDATE subscriptions SET status = $1 WHERE razorpay_subscription_id = $2",
            )
            .bind(new_status)
            .bind(sub_id)
            .execute(&pool)
            .await?;
        }
        _ => {
            tracing::warn!("Unhandled Razorpay event: {}", event_type);
        }
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
}
