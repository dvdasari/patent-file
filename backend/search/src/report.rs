//! PDF report generation for prior art search results.
//!
//! Generates a patentability opinion report suitable for submission
//! with Indian patent applications.

use anyhow::Result;
use printpdf::*;

use crate::models::{PriorArtResult, PriorArtSearch};

const PAGE_WIDTH_MM: f32 = 210.0;
const PAGE_HEIGHT_MM: f32 = 297.0;
const MARGIN_MM: f32 = 25.0;
const LINE_HEIGHT_PT: f32 = 14.0;
const SMALL_LINE_PT: f32 = 11.0;

pub fn generate_search_report(
    search: &PriorArtSearch,
    results: &[PriorArtResult],
) -> Result<Vec<u8>> {
    let (doc, page1, layer1) = PdfDocument::new(
        "Prior Art Search Report",
        Mm(PAGE_WIDTH_MM),
        Mm(PAGE_HEIGHT_MM),
        "Layer 1",
    );

    let font = doc.add_builtin_font(BuiltinFont::TimesRoman)?;
    let bold = doc.add_builtin_font(BuiltinFont::TimesBold)?;
    let italic = doc.add_builtin_font(BuiltinFont::TimesItalic)?;

    let mut current_layer = doc.get_page(page1).get_layer(layer1);
    let mut y = PAGE_HEIGHT_MM - MARGIN_MM;
    let x = MARGIN_MM;
    let content_width = PAGE_WIDTH_MM - 2.0 * MARGIN_MM;

    // Helper closure to add a new page
    let mut page_count = 1;
    let mut new_page = |doc: &PdfDocumentReference, y: &mut f32| -> PdfLayerReference {
        page_count += 1;
        let (page, layer) = doc.add_page(
            Mm(PAGE_WIDTH_MM),
            Mm(PAGE_HEIGHT_MM),
            &format!("Page {page_count}"),
        );
        *y = PAGE_HEIGHT_MM - MARGIN_MM;
        doc.get_page(page).get_layer(layer)
    };

    // Check if we need a new page
    macro_rules! check_page {
        ($needed:expr) => {
            if y < MARGIN_MM + $needed {
                current_layer = new_page(&doc, &mut y);
            }
        };
    }

    // --- Title ---
    current_layer.use_text(
        "PRIOR ART SEARCH REPORT",
        18.0,
        Mm(x),
        Mm(y),
        &bold,
    );
    y -= 8.0;

    current_layer.use_text(
        "Patentability Opinion — Indian Patent Office",
        12.0,
        Mm(x),
        Mm(y),
        &italic,
    );
    y -= 10.0;

    // --- Horizontal rule ---
    let line = Line {
        points: vec![
            (Point::new(Mm(x), Mm(y)), false),
            (Point::new(Mm(x + content_width), Mm(y)), false),
        ],
        is_closed: false,
    };
    current_layer.add_line(line);
    y -= 8.0;

    // --- Search Parameters ---
    current_layer.use_text("Search Parameters", 14.0, Mm(x), Mm(y), &bold);
    y -= 6.0;

    let params = [
        ("Query", search.query_text.as_str()),
        (
            "IPC Classification",
            search.ipc_classification.as_deref().unwrap_or("—"),
        ),
        (
            "Applicant Filter",
            search.applicant_filter.as_deref().unwrap_or("—"),
        ),
        (
            "Date Range",
            &format!(
                "{} to {}",
                search
                    .date_from
                    .map(|d| d.format("%d/%m/%Y").to_string())
                    .unwrap_or_else(|| "Any".to_string()),
                search
                    .date_to
                    .map(|d| d.format("%d/%m/%Y").to_string())
                    .unwrap_or_else(|| "Present".to_string()),
            ),
        ),
        (
            "Non-Patent Literature",
            if search.include_npl { "Included" } else { "Excluded" },
        ),
        ("Total Results", &search.result_count.to_string()),
    ];

    for (label, value) in &params {
        check_page!(5.0);
        let line_text = format!("{label}: {value}");
        // Truncate if too long for one line
        let display = if line_text.len() > 90 {
            format!("{}...", &line_text[..87])
        } else {
            line_text
        };
        current_layer.use_text(&display, 10.0, Mm(x + 2.0), Mm(y), &font);
        y -= 4.5;
    }
    y -= 4.0;

    // --- Results Summary ---
    check_page!(15.0);
    current_layer.use_text("Search Results", 14.0, Mm(x), Mm(y), &bold);
    y -= 7.0;

    if results.is_empty() {
        current_layer.use_text(
            "No prior art found matching the search criteria.",
            11.0,
            Mm(x + 2.0),
            Mm(y),
            &italic,
        );
        y -= 6.0;
    }

    for (i, result) in results.iter().enumerate() {
        check_page!(40.0);

        // Result header
        let score_pct = (result.similarity_score * 100.0) as i32;
        let header = format!(
            "{}. {} (Similarity: {}%)",
            i + 1,
            truncate_str(&result.title, 60),
            score_pct
        );
        current_layer.use_text(&header, 11.0, Mm(x), Mm(y), &bold);
        y -= 5.0;

        // Metadata
        let meta_lines = [
            format!(
                "Source: {} | ID: {}",
                result.source,
                result.external_id.as_deref().unwrap_or("—")
            ),
            format!(
                "Applicant: {} | Filed: {} | Published: {}",
                result.applicant.as_deref().unwrap_or("—"),
                result
                    .filing_date
                    .map(|d| d.format("%d/%m/%Y").to_string())
                    .unwrap_or_else(|| "—".to_string()),
                result
                    .publication_date
                    .map(|d| d.format("%d/%m/%Y").to_string())
                    .unwrap_or_else(|| "—".to_string()),
            ),
            format!(
                "IPC: {}",
                result.ipc_codes.as_deref().unwrap_or("—")
            ),
        ];

        for line_text in &meta_lines {
            check_page!(5.0);
            let display = truncate_str(line_text, 95);
            current_layer.use_text(&display, 9.0, Mm(x + 4.0), Mm(y), &font);
            y -= 4.0;
        }

        // Novelty assessment
        if let Some(ref assessment) = result.novelty_assessment {
            check_page!(12.0);
            current_layer.use_text("Novelty Assessment:", 9.0, Mm(x + 4.0), Mm(y), &bold);
            y -= 4.0;

            // Word-wrap the assessment text
            for wrapped_line in word_wrap(assessment, 100) {
                check_page!(5.0);
                current_layer.use_text(&wrapped_line, 9.0, Mm(x + 6.0), Mm(y), &italic);
                y -= 3.5;
            }
        }

        y -= 4.0;
    }

    // --- Footer / Disclaimer ---
    check_page!(20.0);
    y -= 4.0;
    let footer_line = Line {
        points: vec![
            (Point::new(Mm(x), Mm(y)), false),
            (Point::new(Mm(x + content_width), Mm(y)), false),
        ],
        is_closed: false,
    };
    current_layer.add_line(footer_line);
    y -= 6.0;

    current_layer.use_text("Disclaimer", 11.0, Mm(x), Mm(y), &bold);
    y -= 5.0;

    let disclaimer = "This report is generated by AI-assisted prior art search tools and is \
        intended for informational purposes only. It does not constitute a legal opinion. \
        A qualified patent agent or attorney should review all findings before making filing \
        decisions. Search coverage depends on the availability and completeness of public \
        patent databases at the time of the search.";

    for line_text in word_wrap(disclaimer, 105) {
        check_page!(5.0);
        current_layer.use_text(&line_text, 8.0, Mm(x + 2.0), Mm(y), &font);
        y -= 3.5;
    }

    y -= 4.0;
    check_page!(5.0);
    let generated = format!(
        "Generated: {} | Patent Draft Pro — Prior Art Search Engine",
        search.created_at.format("%d %B %Y, %H:%M UTC")
    );
    current_layer.use_text(&generated, 8.0, Mm(x), Mm(y), &italic);

    let bytes = doc.save_to_bytes()?;
    Ok(bytes)
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

fn word_wrap(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() > max_chars {
            lines.push(current_line);
            current_line = word.to_string();
        } else {
            current_line.push(' ');
            current_line.push_str(word);
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_wrap() {
        let lines = word_wrap("hello world this is a test", 12);
        assert_eq!(lines, vec!["hello world", "this is a", "test"]);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world!", 8), "hello...");
    }
}
