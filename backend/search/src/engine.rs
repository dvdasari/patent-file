//! Search engine that orchestrates InPASS, Google Patents, NPL sources
//! and AI novelty assessment into a unified prior art search pipeline.

use anyhow::Result;
use std::sync::Arc;

use ai::LlmProvider;

use crate::google_patents::GooglePatentsClient;
use crate::inpass::InpassClient;
use crate::models::RawPatentResult;
use crate::novelty::{self, AssessedResult};
use crate::npl::NplClient;

pub struct SearchEngine {
    inpass: InpassClient,
    google_patents: GooglePatentsClient,
    npl: NplClient,
    provider: Arc<dyn LlmProvider>,
}

impl SearchEngine {
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self {
            inpass: InpassClient::new(),
            google_patents: GooglePatentsClient::new(),
            npl: NplClient::new(),
            provider,
        }
    }

    /// Execute a prior art search across all configured sources and
    /// return AI-assessed results sorted by relevance.
    pub async fn search(
        &self,
        query: &str,
        ipc: Option<&str>,
        applicant: Option<&str>,
        date_from: Option<chrono::NaiveDate>,
        date_to: Option<chrono::NaiveDate>,
        include_npl: bool,
    ) -> Result<Vec<AssessedResult>> {
        // Search all sources concurrently
        let inpass_fut = self.inpass.search(query, ipc, applicant, date_from, date_to);
        let google_fut = self.google_patents.search(query, ipc, applicant, date_from, date_to);

        let (inpass_results, google_results) = tokio::join!(inpass_fut, google_fut);

        let mut all_results: Vec<RawPatentResult> = Vec::new();

        match inpass_results {
            Ok(r) => {
                tracing::info!("InPASS: {} results", r.len());
                all_results.extend(r);
            }
            Err(e) => tracing::warn!("InPASS search failed: {e}"),
        }

        match google_results {
            Ok(r) => {
                tracing::info!("Google Patents: {} results", r.len());
                all_results.extend(r);
            }
            Err(e) => tracing::warn!("Google Patents search failed: {e}"),
        }

        if include_npl {
            match self.npl.search(query).await {
                Ok(r) => {
                    tracing::info!("NPL: {} results", r.len());
                    all_results.extend(r);
                }
                Err(e) => tracing::warn!("NPL search failed: {e}"),
            }
        }

        // Deduplicate by title similarity (simple exact-match dedup)
        dedup_results(&mut all_results);

        tracing::info!(
            "Total unique results before AI assessment: {}",
            all_results.len()
        );

        if all_results.is_empty() {
            return Ok(Vec::new());
        }

        // AI novelty assessment
        let assessed = novelty::assess_results(self.provider.as_ref(), query, all_results).await?;

        Ok(assessed)
    }
}

fn dedup_results(results: &mut Vec<RawPatentResult>) {
    let mut seen_ids = std::collections::HashSet::new();
    let mut seen_titles = std::collections::HashSet::new();

    results.retain(|r| {
        // Dedup by external_id if present
        if let Some(ref id) = r.external_id {
            let key = id.to_lowercase().replace(' ', "");
            if !seen_ids.insert(key) {
                return false;
            }
        }
        // Dedup by normalized title
        let title_key = r.title.to_lowercase().trim().to_string();
        seen_titles.insert(title_key)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dedup() {
        let mut results = vec![
            RawPatentResult {
                source: "inpass".into(),
                external_id: Some("IN123".into()),
                title: "Method for X".into(),
                applicant: None,
                filing_date: None,
                publication_date: None,
                ipc_codes: None,
                abstract_text: None,
                url: None,
            },
            RawPatentResult {
                source: "google_patents".into(),
                external_id: Some("IN123".into()),
                title: "Method for X".into(),
                applicant: None,
                filing_date: None,
                publication_date: None,
                ipc_codes: None,
                abstract_text: None,
                url: None,
            },
            RawPatentResult {
                source: "inpass".into(),
                external_id: Some("IN456".into()),
                title: "Different Method".into(),
                applicant: None,
                filing_date: None,
                publication_date: None,
                ipc_codes: None,
                abstract_text: None,
                url: None,
            },
        ];

        dedup_results(&mut results);
        assert_eq!(results.len(), 2);
    }
}
