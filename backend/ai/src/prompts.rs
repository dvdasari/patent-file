use crate::provider::Prompt;

pub const SECTION_ORDER: &[&str] = &[
    "title",
    "field_of_invention",
    "background",
    "summary",
    "detailed_description",
    "claims",
    "abstract",
    "drawings_description",
];

pub fn build_prompt(
    section_type: &str,
    interview_context: &str,
    previous_sections: &str,
    figure_descriptions: &str,
) -> Prompt {
    let system = format!(
        "You are an Indian patent specification drafter experienced with IPO Form 2 format.\n\
         Generate the {} section of a patent specification.\n\
         Use formal patent language with proper antecedent basis.\n\
         Follow Indian Patent Office formatting conventions.\n\
         Output ONLY the section content, no headers or labels.",
        section_type.replace('_', " ")
    );

    let mut user = format!("## Invention Disclosure\n\n{}\n\n", interview_context);

    if !previous_sections.is_empty() {
        user.push_str(&format!(
            "## Previously Generated Sections\n\n{}\n\n",
            previous_sections
        ));
    }

    if !figure_descriptions.is_empty() {
        user.push_str(&format!(
            "## Figure Descriptions\n\n{}\n\n",
            figure_descriptions
        ));
    }

    user.push_str(&format!(
        "Generate the {} section now.",
        section_type.replace('_', " ")
    ));

    Prompt { system, user }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt_includes_context() {
        let prompt = build_prompt(
            "claims",
            "My invention is a widget",
            "Title: Widget\nField: Mechanical",
            "",
        );
        assert!(prompt.system.contains("claims"));
        assert!(prompt.user.contains("My invention is a widget"));
        assert!(prompt.user.contains("Widget"));
    }

    #[test]
    fn test_section_order_count() {
        assert_eq!(SECTION_ORDER.len(), 8);
    }

    #[test]
    fn test_prompt_includes_figures() {
        let prompt = build_prompt("drawings_description", "inv", "", "Fig 1: Overview");
        assert!(prompt.user.contains("Fig 1: Overview"));
    }
}
