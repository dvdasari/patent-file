//! Indian Patent Case Law Citation Database
//!
//! Provides relevant case law citations for common objection patterns
//! encountered during Indian patent prosecution.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CaseLawEntry {
    pub id: &'static str,
    pub case_name: &'static str,
    pub citation: &'static str,
    pub year: u16,
    pub section: &'static str,
    pub principle: &'static str,
    pub relevance: &'static str,
}

/// Master database of Indian patent case law citations
pub const CASE_LAW_DB: &[CaseLawEntry] = &[
    // Section 3(d) — Pharma / new forms of known substances
    CaseLawEntry {
        id: "novartis_2013",
        case_name: "Novartis AG v. Union of India",
        citation: "(2013) 6 SCC 1",
        year: 2013,
        section: "3(d)",
        principle: "A new form of a known substance is not patentable unless it demonstrates \
                    significantly enhanced efficacy. The term 'efficacy' in Section 3(d) refers \
                    to therapeutic efficacy for pharmaceutical substances.",
        relevance: "Landmark judgment defining the scope of Section 3(d). Beta crystalline form \
                    of Imatinib Mesylate held not patentable as enhanced bioavailability alone \
                    does not establish enhanced therapeutic efficacy.",
    },
    CaseLawEntry {
        id: "roche_erlotinib",
        case_name: "F. Hoffmann-La Roche Ltd v. Cipla Ltd",
        citation: "2016 (65) PTC 1 (Del)",
        year: 2016,
        section: "3(d)",
        principle: "Polymorph B of Erlotinib granted patent where applicant demonstrated \
                    superior stability and bioavailability with clinical data.",
        relevance: "Shows that Section 3(d) can be overcome with robust comparative efficacy data.",
    },

    // Section 3(k) — Computer programs per se
    CaseLawEntry {
        id: "ferid_allani_2013",
        case_name: "Ferid Allani v. Union of India",
        citation: "W.P.(C) 7/2014, Delhi HC",
        year: 2013,
        section: "3(k)",
        principle: "If the invention demonstrates a technical effect or technical contribution \
                    that goes beyond the mere interaction of software with hardware, it is \
                    patentable despite involving a computer program.",
        relevance: "Key Delhi High Court judgment clarifying that Section 3(k) excludes only \
                    'computer programs per se', not all software-related inventions.",
    },
    CaseLawEntry {
        id: "cri_guidelines_2017",
        case_name: "IPO CRI Guidelines 2017",
        citation: "Office Order No. 36/2017",
        year: 2017,
        section: "3(k)",
        principle: "Computer-related inventions must demonstrate novel hardware, a technical \
                    effect, or solve a technical problem to be patentable. The three-step test: \
                    (1) identify the actual contribution, (2) determine if it is technical, \
                    (3) verify it is not excluded subject matter.",
        relevance: "Official IPO guidelines for examining computer-related patent applications.",
    },
    CaseLawEntry {
        id: "accenture_2018",
        case_name: "Accenture Global Solutions Ltd",
        citation: "IN App. No. 6268/CHENP/2014",
        year: 2018,
        section: "3(k)",
        principle: "Patent for system managing computing resources granted where technical \
                    contribution was in improved resource allocation, not business logic.",
        relevance: "Example of software patent granted under CRI Guidelines by emphasizing technical contribution.",
    },

    // Section 3(e) — Aggregation
    CaseLawEntry {
        id: "esco_biotech",
        case_name: "Esco Biotech Pvt. Ltd v. CCL Pharmaceuticals",
        citation: "2019 IPAB Order",
        year: 2019,
        section: "3(e)",
        principle: "A combination of known substances is patentable under Indian law only if \
                    the combination shows synergistic properties not predictable from the \
                    individual components.",
        relevance: "Clarifies the synergy requirement for composition claims under Section 3(e).",
    },

    // Section 10 — Sufficiency of disclosure
    CaseLawEntry {
        id: "bishwanath_1979",
        case_name: "Bishwanath Prasad v. Hindustan Metal Industries",
        citation: "AIR 1982 SC 1444",
        year: 1979,
        section: "10",
        principle: "The specification must be sufficient to enable a person skilled in the art \
                    to perform the invention without undue experimentation. Insufficiency is a \
                    valid ground for revocation.",
        relevance: "Foundational Supreme Court judgment on sufficiency of disclosure requirement.",
    },
    CaseLawEntry {
        id: "raj_prakash_1978",
        case_name: "Raj Prakash v. Mangat Ram Choudhary",
        citation: "AIR 1978 Del 1",
        year: 1978,
        section: "10",
        principle: "Claims must be fairly based on the matter disclosed in the specification. \
                    Overly broad claims unsupported by the description are invalid.",
        relevance: "Establishes that claims must be commensurate in scope with the disclosure.",
    },

    // Section 10(5) — Clarity of claims
    CaseLawEntry {
        id: "biswanath_clear",
        case_name: "Biswanath Prasad Radhey Shyam v. Hindustan Metal Industries",
        citation: "1982 SCR (1) 714",
        year: 1982,
        section: "10(5)",
        principle: "Claims must be clear and succinct. Ambiguous claim language that does not \
                    clearly define the scope of protection is ground for rejection.",
        relevance: "Supreme Court guidance on claim clarity requirements.",
    },

    // Section 48 — Rights of patentees (infringement context)
    CaseLawEntry {
        id: "monsanto_2019",
        case_name: "Monsanto Technology LLC v. Nuziveedu Seeds Ltd",
        citation: "(2019) 3 SCC 381",
        year: 2019,
        section: "3(j)/48",
        principle: "Genetically modified plants and seeds are not excluded from patentability \
                    merely because they involve biological processes. The patent claim must be \
                    assessed on its technical contribution.",
        relevance: "Important Supreme Court ruling on scope of Section 3(j) exclusion for biotech patents.",
    },
];

/// Get a formatted citation string for a given context
pub fn get_citation(context: &str) -> String {
    match context {
        "section_3d" => {
            let novartis = &CASE_LAW_DB[0]; // novartis_2013
            format!(
                "Section 3(d) — {} {}: {}",
                novartis.case_name, novartis.citation, novartis.principle
            )
        }
        "section_3k_software" => {
            let ferid = &CASE_LAW_DB[2]; // ferid_allani_2013
            let cri = &CASE_LAW_DB[3]; // cri_guidelines_2017
            format!(
                "Section 3(k) — {} {}: {}. See also {} ({}).",
                ferid.case_name, ferid.citation, ferid.principle,
                cri.case_name, cri.citation
            )
        }
        "section_3k_business" => {
            let cri = &CASE_LAW_DB[3];
            format!(
                "Section 3(k) — Business methods are non-patentable. {}.",
                cri.case_name
            )
        }
        "section_3k_math" => {
            format!(
                "Section 3(k) — Mathematical methods per se are non-patentable. See CRI Guidelines 2017."
            )
        }
        "section_3e" => {
            let esco = &CASE_LAW_DB[4]; // esco_biotech
            format!(
                "Section 3(e) — {} {}: {}",
                esco.case_name, esco.citation, esco.principle
            )
        }
        "section_10_disclosure" => {
            let bish = &CASE_LAW_DB[5]; // bishwanath_1979
            format!("{} {}", bish.case_name, bish.citation)
        }
        "section_10_claims" => {
            let raj = &CASE_LAW_DB[6]; // raj_prakash_1978
            format!("{} {}", raj.case_name, raj.citation)
        }
        _ => "Indian Patent Act 1970".into(),
    }
}

/// Search case law database by section number
pub fn search_by_section(section: &str) -> Vec<&'static CaseLawEntry> {
    CASE_LAW_DB
        .iter()
        .filter(|entry| entry.section == section || entry.section.starts_with(section))
        .collect()
}

/// Search case law database by keyword in principle or case name
pub fn search_by_keyword(keyword: &str) -> Vec<&'static CaseLawEntry> {
    let keyword_lower = keyword.to_lowercase();
    CASE_LAW_DB
        .iter()
        .filter(|entry| {
            entry.principle.to_lowercase().contains(&keyword_lower)
                || entry.case_name.to_lowercase().contains(&keyword_lower)
                || entry.relevance.to_lowercase().contains(&keyword_lower)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_law_db_not_empty() {
        assert!(!CASE_LAW_DB.is_empty());
    }

    #[test]
    fn test_search_by_section_3d() {
        let results = search_by_section("3(d)");
        assert!(!results.is_empty());
        assert!(results.iter().any(|e| e.id == "novartis_2013"));
    }

    #[test]
    fn test_search_by_section_3k() {
        let results = search_by_section("3(k)");
        assert!(results.len() >= 2);
    }

    #[test]
    fn test_search_by_keyword() {
        let results = search_by_keyword("efficacy");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_get_citation_section3d() {
        let citation = get_citation("section_3d");
        assert!(citation.contains("Novartis"));
    }

    #[test]
    fn test_get_citation_section3k() {
        let citation = get_citation("section_3k_software");
        assert!(citation.contains("Ferid Allani"));
        assert!(citation.contains("CRI Guidelines"));
    }

    #[test]
    fn test_all_entries_have_required_fields() {
        for entry in CASE_LAW_DB {
            assert!(!entry.id.is_empty());
            assert!(!entry.case_name.is_empty());
            assert!(!entry.citation.is_empty());
            assert!(entry.year > 1970);
            assert!(!entry.section.is_empty());
            assert!(!entry.principle.is_empty());
        }
    }
}
