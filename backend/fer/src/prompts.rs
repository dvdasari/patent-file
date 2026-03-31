use ai::Prompt;

/// Build prompt to parse a FER and extract objections
pub fn build_parse_prompt(fer_text: &str) -> Prompt {
    Prompt {
        system: r#"You are an expert Indian patent attorney AI assistant specializing in analyzing First Examination Reports (FERs) from the Indian Patent Office (IPO).

Your task is to parse the given FER text and extract structured information. Return a JSON object with the following structure:

{
  "examiner_name": "Name of the examiner if mentioned, or null",
  "objections": [
    {
      "objection_number": 1,
      "category": "one of: novelty, inventive_step, non_patentable, insufficiency, unity, formal, other",
      "section_reference": "The specific section of the Patents Act referenced, e.g. 'Section 2(1)(j)', 'Section 3(d)', or null",
      "summary": "A concise one-line summary of the objection",
      "full_text": "The complete text of this specific objection from the FER"
    }
  ]
}

Category mapping:
- novelty: Objections under Section 2(1)(j) or Section 13 regarding lack of novelty/anticipation
- inventive_step: Objections under Section 2(1)(ja) regarding obviousness/lack of inventive step
- non_patentable: Objections under Section 3 (non-patentable subject matter, e.g., 3(d), 3(e), 3(k))
- insufficiency: Objections under Section 10 regarding insufficient disclosure or lack of enablement
- unity: Objections under Section 10(5) regarding lack of unity of invention
- formal: Formal/procedural objections (e.g., drawings, claims format, priority documents)
- other: Any other objections not fitting the above categories

Return ONLY valid JSON, no other text."#.to_string(),
        user: format!("Parse the following First Examination Report and extract all objections:\n\n{}", fer_text),
    }
}

/// Build prompt to generate a response to a specific FER objection
pub fn build_response_prompt(
    objection: &str,
    category: &str,
    section_ref: Option<&str>,
    fer_context: &str,
) -> Prompt {
    let section_note = section_ref
        .map(|s| format!("\nRelevant section: {}", s))
        .unwrap_or_default();

    Prompt {
        system: format!(
            r#"You are an expert Indian patent attorney AI assistant. Generate a comprehensive response to a First Examination Report (FER) objection from the Indian Patent Office.

The objection category is: {category}{section_note}

Your response must be structured in three clearly marked sections using these exact headings:

## Legal Arguments
Provide detailed legal arguments addressing the objection. Reference specific sections of the Indian Patents Act 1970, the Patent Rules 2003, and TRIPS provisions where applicable. Address the examiner's specific concerns point by point.

## Suggested Claim Amendments
If claim amendments would help overcome the objection, suggest specific language. If no amendments are needed, explain why the claims as filed already address the concern.

## Indian Case Law & Citations
Cite relevant Indian IPO case law, IPAB decisions, and High Court/Supreme Court judgments. Include:
- Case name and citation
- Key holding relevant to this objection
- How it supports the applicant's position

Also cite relevant Manual of Patent Office Practice and Procedure (MPPP) guidelines.

Focus on Indian patent law. Be specific and actionable. The response should be ready for adaptation by a patent agent for filing."#
        ),
        user: format!(
            "FER context:\n{}\n\nSpecific objection to respond to:\n{}",
            fer_context, objection
        ),
    }
}
