use axum::extract::{Query, State};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
}

fn default_page() -> i32 {
    1
}
fn default_per_page() -> i32 {
    10
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PatentResult {
    pub patent_number: String,
    pub title: String,
    pub abstract_text: Option<String>,
    pub applicants: Vec<String>,
    pub inventors: Vec<String>,
    pub filing_date: Option<String>,
    pub publication_date: Option<String>,
    pub jurisdiction: String,
    pub url: Option<String>,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub results: Vec<PatentResult>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub query: String,
}

#[derive(Clone)]
pub struct SearchState {
    pub pool: PgPool,
    pub lens_api_key: Option<String>,
    pub patent_search_provider: String,
}

pub async fn search_patents(
    Extension(_auth): Extension<AuthUser>,
    State(state): State<SearchState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, AppError> {
    let q = params.q.trim().to_string();
    if q.is_empty() {
        return Err(AppError::bad_request("Search query cannot be empty"));
    }

    let per_page = params.per_page.clamp(1, 50);
    let page = params.page.max(1);

    // Compute a stable cache key from (query, page, per_page)
    let cache_input = format!("{}:{}:{}", q.to_lowercase(), page, per_page);
    let mut hasher = Sha256::new();
    hasher.update(cache_input.as_bytes());
    let query_hash = format!("{:x}", hasher.finalize());

    // Serve from cache when available and not expired
    if let Some(cached) = get_cached(&state.pool, &query_hash, page, per_page).await? {
        return Ok(Json(cached));
    }

    // Execute live search
    let (results, total) = match state.patent_search_provider.as_str() {
        "lens" => {
            search_lens(&q, page, per_page, state.lens_api_key.as_deref()).await?
        }
        _ => search_mock(&q, page, per_page),
    };

    // Persist to cache
    store_cache(&state.pool, &query_hash, &q, page, per_page, &results, total).await?;

    Ok(Json(SearchResponse {
        results,
        total,
        page,
        per_page,
        query: q,
    }))
}

// ── Cache helpers ─────────────────────────────────────────────────────────────

async fn get_cached(
    pool: &PgPool,
    query_hash: &str,
    page: i32,
    per_page: i32,
) -> Result<Option<SearchResponse>, AppError> {
    let row: Option<(serde_json::Value, i64, String)> = sqlx::query_as(
        "SELECT results, total, query_text
         FROM patent_search_cache
         WHERE query_hash = $1 AND page = $2 AND per_page = $3
           AND expires_at > now()
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(query_hash)
    .bind(page)
    .bind(per_page)
    .fetch_optional(pool)
    .await?;

    if let Some((results_json, total, query_text)) = row {
        let results: Vec<PatentResult> =
            serde_json::from_value(results_json).unwrap_or_default();
        return Ok(Some(SearchResponse {
            results,
            total,
            page,
            per_page,
            query: query_text,
        }));
    }

    Ok(None)
}

async fn store_cache(
    pool: &PgPool,
    query_hash: &str,
    query_text: &str,
    page: i32,
    per_page: i32,
    results: &[PatentResult],
    total: i64,
) -> Result<(), AppError> {
    let results_json = serde_json::to_value(results)
        .map_err(|e| AppError::internal(format!("Failed to serialise search results: {e}")))?;

    sqlx::query(
        "INSERT INTO patent_search_cache
             (query_hash, query_text, page, per_page, results, total)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(query_hash)
    .bind(query_text)
    .bind(page)
    .bind(per_page)
    .bind(results_json)
    .bind(total)
    .execute(pool)
    .await?;

    Ok(())
}

// ── Lens.org provider ─────────────────────────────────────────────────────────

async fn search_lens(
    q: &str,
    page: i32,
    per_page: i32,
    api_key: Option<&str>,
) -> Result<(Vec<PatentResult>, i64), AppError> {
    let key = api_key
        .ok_or_else(|| AppError::internal("LENS_API_KEY not configured for patent search"))?;

    let from = (page - 1) * per_page;
    let body = serde_json::json!({
        "query": {
            "bool": {
                "must": [{
                    "query_string": {
                        "query": q,
                        "fields": ["title", "abstract", "claims"]
                    }
                }],
                "filter": [
                    { "term": { "jurisdiction": "IN" } }
                ]
            }
        },
        "size": per_page,
        "from": from,
        "include": [
            "lens_id", "pub_key", "title", "abstract", "applicant", "inventor",
            "date_published", "date_filed", "jurisdiction"
        ]
    });

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.lens.org/patent/search")
        .bearer_auth(key)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::internal(format!("Lens API request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        tracing::error!("Lens API returned {}: {}", status, text);
        return Err(AppError::internal("Patent search service unavailable"));
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::internal(format!("Failed to parse Lens response: {e}")))?;

    let total = data["total"].as_i64().unwrap_or(0);
    let mut results = Vec::new();

    if let Some(hits) = data["data"].as_array() {
        for hit in hits {
            let patent_number = hit["pub_key"]
                .as_str()
                .or_else(|| hit["lens_id"].as_str())
                .unwrap_or("UNKNOWN")
                .to_string();

            let title = hit["title"]
                .as_array()
                .and_then(|t| t.first())
                .and_then(|t| t["text"].as_str())
                .unwrap_or("")
                .to_string();

            let abstract_text = hit["abstract"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|a| a["text"].as_str())
                .map(|s| s.to_string());

            let applicants = hit["applicant"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|a| a["name"].as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let inventors = hit["inventor"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|i| i["name"].as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let filing_date = hit["date_filed"].as_str().map(|s| s.to_string());
            let publication_date = hit["date_published"].as_str().map(|s| s.to_string());

            let lens_id = hit["lens_id"].as_str().unwrap_or("").to_string();
            let url = if lens_id.is_empty() {
                None
            } else {
                Some(format!("https://lens.org/lens/patent/{}", lens_id))
            };

            results.push(PatentResult {
                patent_number,
                title,
                abstract_text,
                applicants,
                inventors,
                filing_date,
                publication_date,
                jurisdiction: "IN".to_string(),
                url,
            });
        }
    }

    Ok((results, total))
}

// ── Mock provider (dev / CI) ──────────────────────────────────────────────────

fn search_mock(q: &str, page: i32, per_page: i32) -> (Vec<PatentResult>, i64) {
    let all_results = mock_patents(q);
    let total = all_results.len() as i64;
    let start = ((page - 1) * per_page) as usize;
    let results = all_results
        .into_iter()
        .skip(start)
        .take(per_page as usize)
        .collect();
    (results, total)
}

fn mock_patents(q: &str) -> Vec<PatentResult> {
    let q_cap = {
        let mut c = q.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    };

    vec![
        PatentResult {
            patent_number: "IN202141056789A".to_string(),
            title: format!("{} system with enhanced efficiency and reduced operational complexity", q_cap),
            abstract_text: Some(format!(
                "The present invention discloses a system and method relating to {}. \
                 The invention addresses key technical challenges in the domain by providing \
                 a novel mechanism that improves performance by up to 40% over prior art. \
                 The system comprises a primary processing unit, a feedback control module, \
                 and an adaptive interface layer. Applications include industrial automation, \
                 consumer electronics, and medical devices.",
                q
            )),
            applicants: vec!["Tata Consultancy Services Limited".to_string()],
            inventors: vec!["Ramesh Kumar Singh".to_string(), "Priya Sharma".to_string()],
            filing_date: Some("2021-08-15".to_string()),
            publication_date: Some("2022-02-25".to_string()),
            jurisdiction: "IN".to_string(),
            url: Some("https://ipindiaservices.gov.in/PatentSearch".to_string()),
        },
        PatentResult {
            patent_number: "IN202011034521B".to_string(),
            title: format!("Method and apparatus for {} with real-time processing", q_cap),
            abstract_text: Some(format!(
                "A method and apparatus for real-time {} processing is disclosed. \
                 The invention leverages machine learning to adaptively tune operational \
                 parameters. A neural network classifier is trained on domain-specific data \
                 to achieve high accuracy with low latency. The invention is particularly \
                 suited for embedded systems with constrained resources.",
                q
            )),
            applicants: vec!["Infosys Limited".to_string()],
            inventors: vec!["Anand Krishnamurthy".to_string()],
            filing_date: Some("2020-05-10".to_string()),
            publication_date: Some("2021-11-18".to_string()),
            jurisdiction: "IN".to_string(),
            url: Some("https://ipindiaservices.gov.in/PatentSearch".to_string()),
        },
        PatentResult {
            patent_number: "IN201941023145A".to_string(),
            title: format!("Integrated {} platform with multi-layer security architecture", q_cap),
            abstract_text: Some(format!(
                "An integrated platform for {} is presented, incorporating a multi-layer \
                 security framework compliant with ISO 27001. The platform supports \
                 horizontal scaling across distributed nodes. A novel consensus protocol \
                 ensures data integrity with sub-second finality. The invention has been \
                 validated in production environments handling over 10 million transactions daily.",
                q
            )),
            applicants: vec![
                "Wipro Limited".to_string(),
                "Indian Institute of Technology Bombay".to_string(),
            ],
            inventors: vec![
                "Suresh Patel".to_string(),
                "Meena Iyer".to_string(),
                "Vikram Nair".to_string(),
            ],
            filing_date: Some("2019-11-22".to_string()),
            publication_date: Some("2020-05-07".to_string()),
            jurisdiction: "IN".to_string(),
            url: Some("https://ipindiaservices.gov.in/PatentSearch".to_string()),
        },
        PatentResult {
            patent_number: "IN202241067890A".to_string(),
            title: format!("Adaptive {} controller using reinforcement learning", q_cap),
            abstract_text: Some(format!(
                "This invention proposes an adaptive controller for {} applications \
                 based on deep reinforcement learning. The controller autonomously learns \
                 optimal policies from environmental feedback, eliminating the need for \
                 manual parameter tuning. Experimental results demonstrate a 60% reduction \
                 in convergence time compared to traditional PID controllers.",
                q
            )),
            applicants: vec!["HCL Technologies Limited".to_string()],
            inventors: vec!["Deepak Agarwal".to_string(), "Kavitha Subramanian".to_string()],
            filing_date: Some("2022-03-30".to_string()),
            publication_date: Some("2022-09-16".to_string()),
            jurisdiction: "IN".to_string(),
            url: Some("https://ipindiaservices.gov.in/PatentSearch".to_string()),
        },
        PatentResult {
            patent_number: "IN201841009876B".to_string(),
            title: format!("Low-power {} device for IoT applications", q_cap),
            abstract_text: Some(format!(
                "A low-power device for {} in IoT environments is disclosed. The device \
                 uses a novel duty-cycling scheme to achieve battery life exceeding five years \
                 on a standard CR2032 cell. Communication is handled via a custom LPWAN \
                 protocol optimised for Indian spectrum regulations. The device has been \
                 field-tested across agriculture, smart cities, and healthcare deployments.",
                q
            )),
            applicants: vec!["Bharti Airtel Limited".to_string()],
            inventors: vec!["Rajiv Mehta".to_string()],
            filing_date: Some("2018-07-04".to_string()),
            publication_date: Some("2019-01-11".to_string()),
            jurisdiction: "IN".to_string(),
            url: Some("https://ipindiaservices.gov.in/PatentSearch".to_string()),
        },
        PatentResult {
            patent_number: "IN202341078901A".to_string(),
            title: format!("{} optimisation framework for supply chain management", q_cap),
            abstract_text: Some(format!(
                "A framework for optimising {} in supply chain contexts is presented. \
                 The framework integrates real-time sensor data with predictive analytics \
                 to minimise inventory costs while maintaining service levels. An open API \
                 enables integration with existing ERP systems. Pilot deployments in three \
                 Fortune 500 companies showed 25% cost reduction within six months.",
                q
            )),
            applicants: vec!["Mahindra & Mahindra Limited".to_string()],
            inventors: vec![
                "Sunita Rao".to_string(),
                "Arun Deshmukh".to_string(),
                "Pooja Joshi".to_string(),
            ],
            filing_date: Some("2023-01-17".to_string()),
            publication_date: Some("2023-07-28".to_string()),
            jurisdiction: "IN".to_string(),
            url: Some("https://ipindiaservices.gov.in/PatentSearch".to_string()),
        },
    ]
}
