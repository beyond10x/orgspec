//! Every refusal of `orgspec type check`: one fixture directory each, whose `subject.yaml` is
//! refused for that reason and no other.

use super::common::{fixtures, type_doc};
use orgspec::check::check_type;
use orgspec::typeset::REFUSALS;

/// Every refusal and the subjects its fixture is refused for, in order.
const REFUSED: &[(&str, &[&str])] = &[
    ("unknown_type", &["gallery"]),
    ("type_cycle", &["first"]),
    ("type_declared_twice", &["twin"]),
    ("kind_declared_twice", &["upper.desk", "upper.shelf"]),
    (
        "unknown_block",
        &[
            "museum.hut",
            "museum.huts_are_few",
            "museum.huts_open",
            "museum.museum.ticket",
        ],
    ),
    (
        "kind_unknown_dimension",
        &["museum.crew", "museum.guide", "museum.wing"],
    ),
    ("kind_dimension_conflict", &["museum.council"]),
    ("kind_is_unknown", &["museum.gallery"]),
    ("kind_is_other_block", &["museum.gallery"]),
    ("kind_is_cycle", &["museum.gallery", "museum.wing"]),
    (
        "attribute_shadows_dimension",
        &["museum.gallery.floors", "museum.wing.nature"],
    ),
    ("link_declared_twice", &["upper.curates", "upper.decides"]),
    (
        "rule_declared_twice",
        &["upper.one_board", "upper.shallow", "upper.unknown_block"],
    ),
    (
        "unknown_rule_form",
        &["museum.guides_per_wing", "museum.wings_are_few"],
    ),
    (
        "unknown_key",
        &[
            "museum",
            "museum.asset_has_curator",
            "museum.curates",
            "museum.curator_answers",
            "museum.curator_separate",
            "museum.guides_not_alone",
            "museum.museum.ticket",
            "museum.visitors_per_day",
            "museum.wing.floors",
            "museum.wing_has_floors",
        ],
    ),
    (
        "unknown_carries",
        &["museum.asset_has_curator", "museum.guide_assists"],
    ),
    (
        "unknown_kind",
        &[
            "museum.guide_answers",
            "museum.guided_tours",
            "museum.wings_are_few",
            "museum.wings_open",
        ],
    ),
    ("gate_loosened", &["upper.one_board", "upper.shallow"]),
    (
        "bounds_unsatisfiable",
        &[
            "upper.at_most_one_board",
            "upper.few_curators",
            "upper.forum_weekly_at_most",
            "upper.no_deputies",
            "upper.unit_basis_market",
        ],
    ),
    (
        "kpi_over_party",
        &["museum.hours_per_holder", "museum.member_score"],
    ),
];

#[test]
fn every_refusal_has_a_fixture_that_is_refused_for_it_alone() {
    let listed: Vec<&str> = REFUSED.iter().map(|(refusal, _)| *refusal).collect();
    assert_eq!(listed, REFUSALS);
    for (refusal, subjects) in REFUSED {
        let report = type_doc(&format!("refusals/{refusal}/subject.yaml"));
        let found: Vec<(&str, &str)> = report
            .refusals
            .iter()
            .map(|r| (r.refusal, r.subject.as_str()))
            .collect();
        let expected: Vec<(&str, &str)> = subjects.iter().map(|s| (*refusal, *s)).collect();
        assert_eq!(found, expected, "{:#?}", report.refusals);
        assert!(!report.valid, "{refusal}");
        assert_eq!(report.exit_code(), 1, "{refusal}");
        let lines = report.lines();
        assert_eq!(lines.len(), subjects.len() + 1, "{refusal}");
        assert!(
            lines[0].starts_with(&format!("{refusal} {} ", subjects[0])),
            "{lines:?}"
        );
    }
}

#[test]
fn a_file_that_declares_base_is_refused() {
    let report = type_doc("declares-base/subject.yaml");
    assert_eq!(report.refusals.len(), 1);
    assert_eq!(report.refusals[0].refusal, "type_declared_twice");
    assert_eq!(report.refusals[0].subject, "base");
}

#[test]
fn bounds_are_compared_after_the_selectors_are_put_in_one_form() {
    let report = type_doc("refusals/bounds_unsatisfiable/subject.yaml");
    let message = |subject: &str| {
        let found = report.refusals.iter().find(|r| r.subject == subject);
        found
            .unwrap_or_else(|| panic!("{subject}"))
            .message
            .as_str()
    };
    // Written `{ nature: [governing], block: unit }` here and `{ block: unit, nature: governing }`
    // in the extended type.
    assert!(message("upper.at_most_one_board").contains("lower.two_boards"));
    assert!(message("upper.no_deputies").contains("lower.head_has_deputy"));
    assert_eq!(message("upper.few_curators"), "min 3 is above max 2");
    assert!(message("upper.forum_weekly_at_most").contains("lower.forum_fortnightly_at_least"));
    assert!(message("upper.unit_basis_market").contains("lower.unit_basis_function"));
}

#[test]
fn the_type_a_refused_type_extends_is_accepted_alone() {
    for dir in [
        "bounds_unsatisfiable",
        "gate_loosened",
        "rule_declared_twice",
    ] {
        let report = type_doc(&format!("refusals/{dir}/lower.yaml"));
        assert!(report.valid, "{dir}: {:#?}", report.refusals);
    }
}

#[test]
fn a_type_document_without_the_shape_of_the_format_cannot_be_read() {
    let dir = fixtures().join("unreadable");
    let read = |name: &str| check_type(&dir.join(name), std::slice::from_ref(&dir));
    let error = read("type-missing-key.yaml").unwrap_err();
    assert!(error.contains("rules[0]: missing key `id`"), "{error}");
    let error = read("type-unknown-format.yaml").unwrap_err();
    assert!(error.contains("unknown format `orgspec-type/7`"), "{error}");
    let error = read("not-yaml.yaml").unwrap_err();
    assert!(error.starts_with("not YAML"), "{error}");
}
