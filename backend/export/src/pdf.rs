use crate::PatentDocument;
use printpdf::*;
use std::io::BufWriter;

const PAGE_W: f64 = 210.0; // A4 mm
const PAGE_H: f64 = 297.0;
const MARGIN_LEFT: f64 = 30.0;
const MARGIN_TOP: f64 = 25.0;
const MARGIN_BOTTOM: f64 = 25.0;
const LINE_HEIGHT_BODY: f64 = 5.5;
const LINE_HEIGHT_HEADING: f64 = 8.0;
const FONT_SIZE_BODY: f64 = 12.0;
const FONT_SIZE_HEADING: f64 = 14.0;
const FONT_SIZE_TITLE: f64 = 16.0;
const CHARS_PER_LINE: usize = 80;

/// Generate a PDF document from patent specification data.
pub fn generate_pdf(doc: &PatentDocument) -> anyhow::Result<Vec<u8>> {
    let (pdf_doc, page1, layer1) =
        PdfDocument::new("Patent Specification", Mm(PAGE_W), Mm(PAGE_H), "Page 1");

    let font_regular = pdf_doc.add_builtin_font(BuiltinFont::TimesRoman)?;
    let font_bold = pdf_doc.add_builtin_font(BuiltinFont::TimesBold)?;

    let first_layer = pdf_doc.get_page(page1).get_layer(layer1);

    let mut writer = PdfWriter {
        doc: &pdf_doc,
        font_regular: &font_regular,
        font_bold: &font_bold,
        current_layer: first_layer,
        y: PAGE_H - MARGIN_TOP,
        page_num: 1,
    };

    // Header
    writer.add_centered_text(
        if doc.patent_type == "provisional" {
            "PROVISIONAL SPECIFICATION"
        } else {
            "COMPLETE SPECIFICATION"
        },
        FONT_SIZE_TITLE,
        true,
    );
    writer.advance(3.0);
    writer.add_centered_text("(See section 10; rule 13)", 10.0, false);
    writer.advance(8.0);

    // Title
    writer.add_centered_text(&doc.title, FONT_SIZE_HEADING, true);
    writer.advance(8.0);

    // Applicant info block
    if let Some(ref app) = doc.applicant {
        writer.add_label_value("Applicant", &app.applicant_name);
        writer.add_label_value("Address", &app.applicant_address);
        writer.add_label_value("Nationality", &app.applicant_nationality);
        writer.advance(2.0);
        writer.add_label_value("Inventor", &app.inventor_name);
        writer.add_label_value("Address", &app.inventor_address);
        writer.add_label_value("Nationality", &app.inventor_nationality);
        if let Some(ref name) = app.agent_name {
            let agent_str = match &app.agent_registration_no {
                Some(reg) => format!("{name} (Reg. No. {reg})"),
                None => name.clone(),
            };
            writer.add_label_value("Patent Agent", &agent_str);
        }
        if let Some(ref assignee) = app.assignee_name {
            writer.add_label_value("Assignee", assignee);
        }
        if let Some(ref date) = app.priority_date {
            let mut priority = date.to_string();
            if let Some(ref country) = app.priority_country {
                priority.push_str(&format!(", {country}"));
            }
            if let Some(ref app_no) = app.priority_application_no {
                priority.push_str(&format!(", App. No. {app_no}"));
            }
            writer.add_label_value("Priority", &priority);
        }
        writer.advance(6.0);
    }

    // Preamble
    let preamble = if doc.patent_type == "provisional" {
        "The following specification describes the invention."
    } else {
        "The following specification particularly describes the invention and the method by which it is to be performed."
    };
    writer.add_paragraph(preamble);
    writer.advance(6.0);

    // Sections
    let sections: &[(&str, &str)] = &[
        ("FIELD OF THE INVENTION", &doc.field_of_invention),
        ("BACKGROUND OF THE INVENTION", &doc.background),
        ("SUMMARY OF THE INVENTION", &doc.summary),
        ("DETAILED DESCRIPTION", &doc.detailed_description),
        ("CLAIMS", &doc.claims),
        ("ABSTRACT", &doc.abstract_text),
        ("DESCRIPTION OF DRAWINGS", &doc.drawings_description),
    ];

    for (heading, content) in sections {
        writer.ensure_space(LINE_HEIGHT_HEADING + LINE_HEIGHT_BODY * 3.0);
        writer.add_heading(heading);
        writer.advance(3.0);
        writer.add_paragraph(content);
        writer.advance(6.0);
    }

    // Save to bytes
    let mut bytes = Vec::new();
    pdf_doc.save(&mut BufWriter::new(&mut bytes))?;
    Ok(bytes)
}

struct PdfWriter<'a> {
    doc: &'a PdfDocumentReference,
    font_regular: &'a IndirectFontRef,
    font_bold: &'a IndirectFontRef,
    current_layer: PdfLayerReference,
    y: f64,
    page_num: u32,
}

impl<'a> PdfWriter<'a> {
    fn new_page(&mut self) {
        self.page_num += 1;
        let (page, layer) = self.doc.add_page(
            Mm(PAGE_W),
            Mm(PAGE_H),
            format!("Page {}", self.page_num),
        );
        self.current_layer = self.doc.get_page(page).get_layer(layer);
        self.y = PAGE_H - MARGIN_TOP;
    }

    fn ensure_space(&mut self, needed_mm: f64) {
        if self.y - needed_mm < MARGIN_BOTTOM {
            self.new_page();
        }
    }

    fn advance(&mut self, mm: f64) {
        self.y -= mm;
        if self.y < MARGIN_BOTTOM {
            self.new_page();
        }
    }

    fn write_line(&mut self, text: &str, font_size: f64, bold: bool) {
        self.ensure_space(LINE_HEIGHT_BODY);
        let font = if bold {
            self.font_bold
        } else {
            self.font_regular
        };
        self.current_layer
            .use_text(text, font_size, Mm(MARGIN_LEFT), Mm(self.y), font);
        self.y -= LINE_HEIGHT_BODY;
    }

    fn add_centered_text(&mut self, text: &str, font_size: f64, bold: bool) {
        self.ensure_space(LINE_HEIGHT_HEADING);
        let font = if bold {
            self.font_bold
        } else {
            self.font_regular
        };
        // Approximate centering: ~0.5 * font_size pt per char ≈ 0.18mm per char
        let text_width_mm = text.len() as f64 * font_size * 0.18;
        let x = ((PAGE_W - text_width_mm) / 2.0).max(MARGIN_LEFT);
        self.current_layer
            .use_text(text, font_size, Mm(x), Mm(self.y), font);
        self.y -= LINE_HEIGHT_HEADING;
    }

    fn add_heading(&mut self, text: &str) {
        self.advance(3.0);
        self.write_line(text, FONT_SIZE_HEADING, true);
        self.advance(2.0);
    }

    fn add_label_value(&mut self, label: &str, value: &str) {
        let line = format!("{label}: {value}");
        for wrapped in wrap_text(&line, CHARS_PER_LINE) {
            self.write_line(&wrapped, FONT_SIZE_BODY, false);
        }
    }

    fn add_paragraph(&mut self, text: &str) {
        for raw_line in text.lines() {
            if raw_line.trim().is_empty() {
                self.advance(LINE_HEIGHT_BODY);
                continue;
            }
            for wrapped in wrap_text(raw_line, CHARS_PER_LINE) {
                self.write_line(&wrapped, FONT_SIZE_BODY, false);
            }
        }
    }
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() > max_chars {
            lines.push(current);
            current = word.to_string();
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_text() {
        let lines = wrap_text("hello world this is a test", 12);
        assert_eq!(lines, vec!["hello world", "this is a", "test"]);
    }

    #[test]
    fn test_wrap_empty() {
        let lines = wrap_text("", 80);
        assert_eq!(lines, vec![""]);
    }

    #[test]
    fn test_generate_pdf_smoke() {
        let doc = PatentDocument {
            title: "Test Patent".to_string(),
            field_of_invention: "Testing".to_string(),
            background: "Background info.".to_string(),
            summary: "Summary info.".to_string(),
            detailed_description: "Details here.".to_string(),
            claims: "1. A method for testing.".to_string(),
            abstract_text: "An abstract.".to_string(),
            drawings_description: "Fig 1 shows test.".to_string(),
            applicant: None,
            patent_type: "complete".to_string(),
        };

        let bytes = generate_pdf(&doc).unwrap();
        assert!(bytes.len() > 100);
        assert_eq!(&bytes[0..5], b"%PDF-");
    }
}
