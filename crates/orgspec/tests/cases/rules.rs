//! Every rule form and the default: one design that holds it and one that breaks it.

use super::common::{design, type_doc, violated, yaml_files};

/// The fixture name of a rule form, and the rule and subject of every violation of the design
/// that breaks it.
const FORMS: &[(&str, &[(&str, &str)])] = &[
    (
        "parts",
        &[
            ("line_unit_staffed", "stacks"),
            ("one_governing_unit", "library"),
        ],
    ),
    ("links", &[("offering_has_responsible", "lending")]),
    (
        "forbid",
        &[("agent_decides_only_reversible", "sorter->disposal:decides")],
    ),
    ("separate", &[("proposer_does_not_approve", "head")]),
    (
        "within",
        &[
            ("borrow_within_parent", "keeper->head:borrows_from"),
            ("refer_within_organization", "helper->head:refers_to"),
            ("stand_in_within_unit", "stacker->head:stands_in"),
        ],
    ),
    (
        "span-depth",
        &[
            ("depth_limited", "junior"),
            ("span_in_seats_limited", "head"),
            ("span_limited", "head"),
        ],
    ),
    (
        "value",
        &[
            ("agent_share_bounded", "sorters"),
            ("forum_meets_regularly", "weekly"),
            ("forum_meets_regularly", "yearly"),
            ("line_unit_basis", "desk"),
        ],
    ),
];

#[test]
fn every_rule_form_is_held_by_one_design_and_broken_by_another() {
    for (form, expected) in FORMS {
        let holds = design(&format!("rules/designs/{form}-holds.yaml"));
        assert!(holds.valid, "{form}: {:#?}", holds.violations);
        assert!(holds.violations.is_empty(), "{form}");
        for (rule, _) in *expected {
            assert_eq!(holds.kpis.get(*rule), Some(&0), "{form}: kpi {rule}");
        }
        let breaks = design(&format!("rules/designs/{form}-breaks.yaml"));
        assert_eq!(
            violated(&breaks),
            *expected,
            "{form}: {:#?}",
            breaks.violations
        );
        assert!(!breaks.valid, "{form}");
        assert_eq!(breaks.exit_code(), 1, "{form}");
        for (rule, _) in *expected {
            let count = expected.iter().filter(|(r, _)| r == rule).count();
            assert_eq!(breaks.kpis.get(*rule), Some(&count), "{form}: kpi {rule}");
        }
    }
}

#[test]
fn every_design_of_the_rule_fixtures_is_one_of_a_pair() {
    assert_eq!(yaml_files("rules/designs").len(), 2 * (FORMS.len() + 1));
}

#[test]
fn the_messages_say_what_was_counted() {
    let message = |file: &str, rule: &str| {
        let report = design(file);
        let found = report.violations.iter().find(|v| v.rule == rule);
        found.unwrap_or_else(|| panic!("{rule}")).message.clone()
    };
    let parts = message("rules/designs/parts-breaks.yaml", "one_governing_unit");
    assert!(parts.contains("has 2 part(s)"), "{parts}");
    assert!(parts.contains("more than max 1"), "{parts}");
    let span = message(
        "rules/designs/span-depth-breaks.yaml",
        "span_in_seats_limited",
    );
    assert!(span.starts_with("7 seat(s) report to it"), "{span}");
    let depth = message("rules/designs/span-depth-breaks.yaml", "depth_limited");
    assert!(depth.contains("has 3 links"), "{depth}");
    let value = message("rules/designs/value-breaks.yaml", "agent_share_bounded");
    assert_eq!(value, "`agent_share: 0.9` is above max 0.75");
}

#[test]
fn a_default_adds_the_link_the_design_left_unsaid() {
    let report = design("rules/designs/default-holds.yaml");
    assert!(report.valid, "{:#?}", report.violations);
    assert_eq!(
        report.defaults["steward_answers"],
        ["sorter->head:answers_to", "fetcher->warden:answers_to"]
    );
    assert_eq!(report.links, 8);
    assert_eq!(report.kpis["agent_without_accountable_human"], 0);
}

#[test]
fn a_default_adds_nothing_when_two_parts_match_and_the_rule_reports() {
    let report = design("rules/designs/default-breaks.yaml");
    assert_eq!(
        violated(&report),
        [("agent_without_accountable_human", "sorter")]
    );
    assert!(report.defaults["steward_answers"].is_empty());
    assert_eq!(report.links, 3);
    assert_eq!(report.exit_code(), 1);
}

#[test]
fn every_type_of_the_fixtures_is_accepted() {
    let dirs = [
        "accepted",
        "composition/types",
        "rules/types",
        "structure/types",
        "typepath/shelf",
    ];
    let mut checked = 0;
    for file in dirs.iter().flat_map(|dir| yaml_files(dir)) {
        if file.ends_with("notes.yaml") {
            continue;
        }
        let report = type_doc(&file);
        assert!(report.valid, "{file}: {:#?}", report.refusals);
        assert_eq!(report.exit_code(), 0, "{file}");
        checked += 1;
    }
    assert_eq!(checked, 15);
}

#[test]
fn a_type_that_uses_every_key_is_read() {
    let report = type_doc("accepted/full.yaml");
    assert_eq!(report.name, "full");
    assert_eq!(report.types, ["base", "lower", "full"]);
    assert_eq!(
        (
            report.kinds,
            report.links,
            report.rules,
            report.defaults,
            report.kpis
        ),
        (2, 1, 4, 1, 2)
    );
}
