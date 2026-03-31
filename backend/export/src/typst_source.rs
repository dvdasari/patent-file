use crate::PatentDocument;

/// Generate typst markup source for the patent specification.
/// This can be compiled to PDF via `typst compile` CLI when available.
pub fn generate_typst_source(doc: &PatentDocument) -> String {
    let mut out = String::with_capacity(8192);

    // Page setup for A4, IPO-standard margins
    out.push_str(
        r#"#set page(paper: "a4", margin: (top: 2.5cm, bottom: 2.5cm, left: 3cm, right: 2.5cm))
#set text(font: "New Computer Modern", size: 12pt)
#set par(justify: true, leading: 0.65em)
#set heading(numbering: none)

"#,
    );

    // Header
    out.push_str("#align(center)[\n");
    if doc.patent_type == "provisional" {
        out.push_str("  #text(size: 14pt, weight: \"bold\")[PROVISIONAL SPECIFICATION]\n");
    } else {
        out.push_str("  #text(size: 14pt, weight: \"bold\")[COMPLETE SPECIFICATION]\n");
    }
    out.push_str("  #v(0.5cm)\n");
    out.push_str("  #text(size: 11pt)[(See section 10; rule 13)]\n");
    out.push_str("]\n\n");

    // Title
    out.push_str("#v(1cm)\n");
    out.push_str("#align(center)[\n");
    out.push_str(&format!(
        "  #text(size: 14pt, weight: \"bold\")[{}]\n",
        escape_typst(&doc.title)
    ));
    out.push_str("]\n\n");

    // Applicant info
    if let Some(ref app) = doc.applicant {
        out.push_str("#v(0.5cm)\n");
        out.push_str("#block(width: 100%, stroke: 0.5pt, inset: 12pt)[\n");
        out.push_str(&format!(
            "  *Applicant:* {} \\\n  *Address:* {} \\\n  *Nationality:* {} \\\n",
            escape_typst(&app.applicant_name),
            escape_typst(&app.applicant_address),
            escape_typst(&app.applicant_nationality),
        ));
        out.push_str(&format!(
            "  *Inventor:* {} \\\n  *Address:* {} \\\n  *Nationality:* {} \\\n",
            escape_typst(&app.inventor_name),
            escape_typst(&app.inventor_address),
            escape_typst(&app.inventor_nationality),
        ));
        if let Some(ref name) = app.agent_name {
            out.push_str(&format!("  *Patent Agent:* {}", escape_typst(name)));
            if let Some(ref reg) = app.agent_registration_no {
                out.push_str(&format!(" (Reg. No. {})", escape_typst(reg)));
            }
            out.push_str(" \\\n");
        }
        if let Some(ref assignee) = app.assignee_name {
            out.push_str(&format!(
                "  *Assignee:* {} \\\n",
                escape_typst(assignee)
            ));
        }
        if let Some(ref date) = app.priority_date {
            out.push_str(&format!("  *Priority Date:* {}", date));
            if let Some(ref country) = app.priority_country {
                out.push_str(&format!(", *Country:* {}", escape_typst(country)));
            }
            if let Some(ref app_no) = app.priority_application_no {
                out.push_str(&format!(", *Application No:* {}", escape_typst(app_no)));
            }
            out.push_str(" \\\n");
        }
        out.push_str("]\n\n");
    }

    // Preamble
    out.push_str("#v(0.5cm)\n");
    out.push_str("The following specification ");
    if doc.patent_type == "provisional" {
        out.push_str("describes the invention.\n\n");
    } else {
        out.push_str(
            "particularly describes the invention and the method by which it is to be performed.\n\n",
        );
    }

    // Sections
    let sections = [
        ("FIELD OF THE INVENTION", &doc.field_of_invention),
        ("BACKGROUND OF THE INVENTION", &doc.background),
        ("SUMMARY OF THE INVENTION", &doc.summary),
        ("DETAILED DESCRIPTION", &doc.detailed_description),
        ("CLAIMS", &doc.claims),
        ("ABSTRACT", &doc.abstract_text),
        ("DESCRIPTION OF DRAWINGS", &doc.drawings_description),
    ];

    for (heading, content) in &sections {
        out.push_str(&format!("\n= {heading}\n\n"));
        out.push_str(&escape_typst(content));
        out.push_str("\n");
        if *heading == "CLAIMS" || *heading == "ABSTRACT" {
            out.push_str("\n#pagebreak()\n");
        }
    }

    out
}

fn escape_typst(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('#', "\\#")
        .replace('$', "\\$")
        .replace('@', "\\@")
        .replace('<', "\\<")
        .replace('>', "\\>")
}
