//! Non-patent literature (NPL) search for Indian scientific publications
//! and CSIR databases.
//!
//! Searches multiple Indian NPL sources:
//! - CSIR (Council of Scientific & Industrial Research) publications
//! - Indian Journal of Technology abstracts via CrossRef

use anyhow::{Context, Result};

use crate::models::RawPatentResult;

const CROSSREF_API: &str = "https://api.crossref.org/works";

#[derive(Debug, Clone)]
pub struct NplClient {
    client: reqwest::Client,
}

#[derive(Debug, serde::Deserialize)]
struct CrossRefResponse {
    #[serde(default)]
    message: CrossRefMessage,
}

#[derive(Debug, Default, serde::Deserialize)]
struct CrossRefMessage {
    #[serde(default)]
    items: Vec<CrossRefItem>,
}

#[derive(Debug, serde::Deserialize)]
struct CrossRefItem {
    #[serde(default, rename = "DOI")]
    doi: Option<String>,
    #[serde(default)]
    title: Vec<String>,
    #[serde(default)]
    author: Vec<CrossRefAuthor>,
    #[serde(default, rename = "published-print")]
    published_print: Option<CrossRefDate>,
    #[serde(default, rename = "abstract")]
    abstract_text: Option<String>,
    #[serde(default)]
    subject: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct CrossRefAuthor {
    #[serde(default)]
    given: Option<String>,
    #[serde(default)]
    family: Option<String>,
    #[serde(default)]
    affiliation: Vec<CrossRefAffiliation>,
}

#[derive(Debug, serde::Deserialize)]
struct CrossRefAffiliation {
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct CrossRefDate {
    #[serde(default, rename = "date-parts")]
    date_parts: Vec<Vec<i32>>,
}

impl NplClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("failed to build HTTP client");
        Self { client }
    }

    /// Search CrossRef for Indian scientific publications related to the query.
    pub async fn search(&self, query: &str) -> Result<Vec<RawPatentResult>> {
        // Focus on Indian institutions and CSIR publications
        let extended_query = format!("{query} CSIR India");

        let params = [
            ("query", extended_query.as_str()),
            ("rows", "20"),
            ("sort", "relevance"),
            (
                "filter",
                "from-pub-date:2000-01-01",
            ),
        ];

        let response = self
            .client
            .get(CROSSREF_API)
            .query(&params)
            .header("User-Agent", "PatentDraftPro/1.0 (mailto:support@patentdraftpro.com)")
            .send()
            .await
            .context("Failed to reach CrossRef API")?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::warn!("CrossRef returned {status}");
            return Ok(Vec::new());
        }

        let parsed: CrossRefResponse = response.json().await.unwrap_or(CrossRefResponse {
            message: CrossRefMessage::default(),
        });

        let results: Vec<RawPatentResult> = parsed
            .message
            .items
            .into_iter()
            .filter_map(|item| {
                let title = item.title.into_iter().next().filter(|t| !t.is_empty())?;
                let authors: Vec<String> = item
                    .author
                    .iter()
                    .map(|a| {
                        format!(
                            "{} {}",
                            a.given.as_deref().unwrap_or(""),
                            a.family.as_deref().unwrap_or("")
                        )
                        .trim()
                        .to_string()
                    })
                    .collect();

                let pub_date = item.published_print.and_then(|d| {
                    let parts = d.date_parts.into_iter().next()?;
                    if parts.len() >= 3 {
                        chrono::NaiveDate::from_ymd_opt(parts[0], parts[1] as u32, parts[2] as u32)
                    } else if parts.len() == 2 {
                        chrono::NaiveDate::from_ymd_opt(parts[0], parts[1] as u32, 1)
                    } else if parts.len() == 1 {
                        chrono::NaiveDate::from_ymd_opt(parts[0], 1, 1)
                    } else {
                        None
                    }
                });

                // Strip XML/HTML tags from abstract
                let clean_abstract = item.abstract_text.map(|a| {
                    let re_open = a.replace("<jats:p>", "").replace("</jats:p>", "");
                    re_open
                        .replace("<p>", "")
                        .replace("</p>", "")
                        .replace("<jats:italic>", "")
                        .replace("</jats:italic>", "")
                });

                Some(RawPatentResult {
                    source: "npl".to_string(),
                    external_id: item.doi.clone(),
                    title,
                    applicant: if authors.is_empty() {
                        None
                    } else {
                        Some(authors.join(", "))
                    },
                    filing_date: None,
                    publication_date: pub_date,
                    ipc_codes: if item.subject.is_empty() {
                        None
                    } else {
                        Some(item.subject.join(", "))
                    },
                    abstract_text: clean_abstract,
                    url: item
                        .doi
                        .map(|d| format!("https://doi.org/{d}")),
                })
            })
            .collect();

        tracing::info!("CrossRef NPL search returned {} results", results.len());
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let _client = NplClient::new();
    }
}
