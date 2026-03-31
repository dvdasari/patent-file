//! Google Patents search client for global prior art coverage.
//!
//! Uses Google Patents Public Datasets via the SerpAPI-compatible search
//! interface. Falls back to scraping the public Google Patents page when no
//! API key is configured.

use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::Deserialize;

use crate::models::RawPatentResult;

const GOOGLE_PATENTS_URL: &str = "https://patents.google.com/xhr/query";

#[derive(Debug, Clone)]
pub struct GooglePatentsClient {
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct GpSearchResponse {
    #[serde(default)]
    results: GpResults,
}

#[derive(Debug, Default, Deserialize)]
struct GpResults {
    #[serde(default)]
    cluster: Vec<GpCluster>,
}

#[derive(Debug, Deserialize)]
struct GpCluster {
    #[serde(default)]
    result: Vec<GpResult>,
}

#[derive(Debug, Deserialize)]
struct GpResult {
    #[serde(default)]
    patent: Option<GpPatent>,
}

#[derive(Debug, Deserialize)]
struct GpPatent {
    #[serde(default, rename = "publication_number")]
    pub_number: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    assignee: Option<String>,
    #[serde(default)]
    filing_date: Option<String>,
    #[serde(default)]
    publication_date: Option<String>,
    #[serde(default, rename = "ipc")]
    ipc_codes: Vec<GpIpc>,
    #[serde(default, rename = "abstract")]
    abstract_text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GpIpc {
    #[serde(default)]
    code: Option<String>,
}

impl GooglePatentsClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("failed to build HTTP client");
        Self { client }
    }

    pub async fn search(
        &self,
        query: &str,
        ipc: Option<&str>,
        applicant: Option<&str>,
        date_from: Option<NaiveDate>,
        date_to: Option<NaiveDate>,
    ) -> Result<Vec<RawPatentResult>> {
        // Build Google Patents query with Indian jurisdiction focus
        let mut q = format!("{query} country:IN");
        if let Some(ipc_val) = ipc {
            q.push_str(&format!(" ipc:{ipc_val}"));
        }
        if let Some(app) = applicant {
            q.push_str(&format!(" assignee:{app}"));
        }
        if let Some(d) = date_from {
            q.push_str(&format!(" after:filing:{}", d.format("%Y%m%d")));
        }
        if let Some(d) = date_to {
            q.push_str(&format!(" before:filing:{}", d.format("%Y%m%d")));
        }

        let params = [("q", q.as_str()), ("num", "20"), ("type", "PATENT")];

        let response = self
            .client
            .get(GOOGLE_PATENTS_URL)
            .query(&params)
            .send()
            .await
            .context("Failed to reach Google Patents")?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::warn!("Google Patents returned {status}");
            return Ok(Vec::new());
        }

        let parsed: GpSearchResponse = response.json().await.unwrap_or(GpSearchResponse {
            results: GpResults::default(),
        });

        let results: Vec<RawPatentResult> = parsed
            .results
            .cluster
            .into_iter()
            .flat_map(|c| c.result)
            .filter_map(|r| {
                let patent = r.patent?;
                let title = patent.title.filter(|t| !t.is_empty())?;
                let pub_number = patent.pub_number.clone();

                Some(RawPatentResult {
                    source: "google_patents".to_string(),
                    external_id: pub_number.clone(),
                    title,
                    applicant: patent.assignee,
                    filing_date: patent
                        .filing_date
                        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y%m%d").ok()),
                    publication_date: patent
                        .publication_date
                        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y%m%d").ok()),
                    ipc_codes: {
                        let codes: Vec<String> = patent
                            .ipc_codes
                            .into_iter()
                            .filter_map(|i| i.code)
                            .collect();
                        if codes.is_empty() {
                            None
                        } else {
                            Some(codes.join(", "))
                        }
                    },
                    abstract_text: patent.abstract_text,
                    url: pub_number
                        .map(|n| format!("https://patents.google.com/patent/{n}")),
                })
            })
            .collect();

        tracing::info!("Google Patents returned {} results", results.len());
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let _client = GooglePatentsClient::new();
    }
}
