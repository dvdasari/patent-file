//! InPASS (Indian Patent Advanced Search System) client.
//!
//! Searches the Indian Patent Office's public InPASS database at
//! https://iprsearch.ipindia.gov.in/ for granted patents and published
//! applications.

use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::Deserialize;

use crate::models::RawPatentResult;

const INPASS_SEARCH_URL: &str = "https://iprsearch.ipindia.gov.in/RQStatus/PatentSearch";
const INPASS_APP_SEARCH_URL: &str =
    "https://iprsearch.ipindia.gov.in/RQStatus/ApplicationSearch";

#[derive(Debug, Clone)]
pub struct InpassClient {
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct InpassResponse {
    #[serde(default)]
    data: Vec<InpassRecord>,
    #[serde(default, rename = "recordsTotal")]
    records_total: i64,
}

#[derive(Debug, Deserialize)]
struct InpassRecord {
    #[serde(default, rename = "applicationNumber")]
    application_number: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default, rename = "applicantName")]
    applicant_name: Option<String>,
    #[serde(default, rename = "filingDate")]
    filing_date: Option<String>,
    #[serde(default, rename = "publicationDate")]
    publication_date: Option<String>,
    #[serde(default, rename = "ipcClassification")]
    ipc_classification: Option<String>,
    #[serde(default, rename = "abstract")]
    abstract_text: Option<String>,
}

impl InpassClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("failed to build HTTP client");
        Self { client }
    }

    /// Search InPASS for Indian patents matching the given criteria.
    pub async fn search(
        &self,
        query: &str,
        ipc: Option<&str>,
        applicant: Option<&str>,
        date_from: Option<NaiveDate>,
        date_to: Option<NaiveDate>,
    ) -> Result<Vec<RawPatentResult>> {
        let mut results = Vec::new();

        // Search granted patents
        let granted = self
            .search_endpoint(INPASS_SEARCH_URL, query, ipc, applicant, date_from, date_to)
            .await
            .context("InPASS granted patent search failed")?;
        results.extend(granted);

        // Search published applications
        let applications = self
            .search_endpoint(
                INPASS_APP_SEARCH_URL,
                query,
                ipc,
                applicant,
                date_from,
                date_to,
            )
            .await
            .context("InPASS application search failed")?;
        results.extend(applications);

        Ok(results)
    }

    async fn search_endpoint(
        &self,
        url: &str,
        query: &str,
        ipc: Option<&str>,
        applicant: Option<&str>,
        date_from: Option<NaiveDate>,
        date_to: Option<NaiveDate>,
    ) -> Result<Vec<RawPatentResult>> {
        let mut form = vec![
            ("searchQuery", query.to_string()),
            ("start", "0".to_string()),
            ("length", "50".to_string()),
        ];

        if let Some(ipc_val) = ipc {
            form.push(("ipcClassification", ipc_val.to_string()));
        }
        if let Some(app) = applicant {
            form.push(("applicantName", app.to_string()));
        }
        if let Some(d) = date_from {
            form.push(("dateFrom", d.format("%d/%m/%Y").to_string()));
        }
        if let Some(d) = date_to {
            form.push(("dateTo", d.format("%d/%m/%Y").to_string()));
        }

        let response = self
            .client
            .post(url)
            .form(&form)
            .send()
            .await
            .context("Failed to reach InPASS")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::warn!("InPASS returned {status}: {body}");
            // Fallback to empty results instead of failing the whole search
            return Ok(Vec::new());
        }

        let parsed: InpassResponse = response.json().await.unwrap_or(InpassResponse {
            data: vec![],
            records_total: 0,
        });

        tracing::info!(
            "InPASS returned {} results (total: {})",
            parsed.data.len(),
            parsed.records_total
        );

        Ok(parsed
            .data
            .into_iter()
            .filter_map(|r| {
                let title = r.title.filter(|t| !t.is_empty())?;
                Some(RawPatentResult {
                    source: "inpass".to_string(),
                    external_id: r.application_number.clone(),
                    title,
                    applicant: r.applicant_name,
                    filing_date: r
                        .filing_date
                        .and_then(|d| NaiveDate::parse_from_str(&d, "%d/%m/%Y").ok()),
                    publication_date: r
                        .publication_date
                        .and_then(|d| NaiveDate::parse_from_str(&d, "%d/%m/%Y").ok()),
                    ipc_codes: r.ipc_classification,
                    abstract_text: r.abstract_text,
                    url: r
                        .application_number
                        .as_ref()
                        .map(|n| format!("https://iprsearch.ipindia.gov.in/RQStatus/PatentCertificate/{n}")),
                })
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let _client = InpassClient::new();
    }
}
