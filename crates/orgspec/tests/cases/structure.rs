//! The structural rules: one design that is valid, and one broken design per rule that reports
//! that rule and no other gate.

use super::common::{design, fixtures, violated, yaml_files};
use orgspec::blocks;
use orgspec::node::{self, Node};
use orgspec::structure;
use std::collections::BTreeSet;

/// Every structural rule and the subjects its broken fixture reports, in order.
const BROKEN: &[(&str, &[&str])] = &[
    ("unknown_block", &["shed"]),
    ("unknown_carries", &["head->board:salutes"]),
    ("unknown_link_parts", &["head->cellar:informed"]),
    ("part_outside_organization", &["kitchen"]),
    ("unknown_role", &["head"]),
    ("reporting_cycle", &["head"]),
    ("subject_without_accountable", &["bread"]),
    ("subject_with_two_accountables", &["bread"]),
    ("accountable_not_position", &["kitchen->bread:accountable"]),
    ("agent_without_accountable_human", &["mixer", "packer"]),
    ("class_without_decider", &["recipe"]),
    ("class_with_two_deciders", &["recipe"]),
    ("decider_not_allowed", &["kitchen->recipe:decides"]),
    ("delegation_without_right", &["head->second:delegates"]),
    ("delegation_exceeds_limit", &["head->second:delegates"]),
    ("demand_without_offering", &["street->kitchen:demand"]),
    ("escalation_dead_end", &["board", "head"]),
    ("border_without_part", &["gate"]),
    ("unreachable_parts", &["miller"]),
    ("unknown_attribute", &["head", "head->board:reports_to"]),
    ("missing_attribute", &["stone_oven"]),
    (
        "bad_value",
        &[
            "bakery",
            "head",
            "head",
            "head",
            "head->board:reports_to",
            "head->board:reports_to",
            "head->board:reports_to",
        ],
    ),
    ("unknown_kind", &["kitchen"]),
    ("kind_block_mismatch", &["kitchen"]),
    ("kind_dimension_mismatch", &["front"]),
    (
        "unknown_reference",
        &[
            "baking",
            "head->baking:accountable",
            "head->baking:accountable",
            "head->plan:accountable",
            "plan",
        ],
    ),
];

#[test]
fn the_valid_design_breaks_no_rule() {
    let report = design("structure/designs/valid.yaml");
    assert!(report.valid, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
    assert_eq!(report.exit_code(), 0);
    assert_eq!(report.organization, "bakery");
    assert_eq!(report.types, ["base", "shop"]);
    assert_eq!((report.parts, report.links), (25, 30));
    for rule in structure::RULES {
        assert_eq!(report.kpis.get(*rule), Some(&0), "{rule}");
    }
    let m = &report.measures;
    assert_eq!((m.positions, m.seats, m.units), (3, 5, 3));
    assert_eq!(m.agent_seats, 2.5);
    assert_eq!((m.span_max, m.reporting_depth_max), (2, 2));
    assert_eq!((m.subjects, m.subjects_with_accountable), (8, 8));
}

#[test]
fn the_valid_design_uses_every_block_and_every_link() {
    let doc = node::load(&fixtures().join("structure/designs/valid.yaml")).unwrap();
    let texts = |nodes: Vec<&Node>, key: &str| -> BTreeSet<String> {
        nodes
            .iter()
            .filter_map(|n| n.get(key).map(Node::text))
            .collect()
    };
    let parts = doc.get("parts").expect("parts").entries();
    let used = texts(parts.iter().map(|(_, part)| part).collect(), "block");
    for block in blocks::BLOCKS {
        assert!(used.contains(block.name), "no part of block {}", block.name);
    }
    let Some(Node::Seq(links)) = doc.get("links") else {
        panic!("links is a list");
    };
    let carried = texts(links.iter().collect(), "carries");
    for link in blocks::LINKS {
        assert!(carried.contains(*link), "no link carries {link}");
    }
}

#[test]
fn every_structural_rule_has_a_fixture_that_breaks_it_and_no_other_gate() {
    let listed: Vec<&str> = BROKEN.iter().map(|(rule, _)| *rule).collect();
    assert_eq!(listed, structure::RULES);
    for (rule, subjects) in BROKEN {
        let report = design(&format!("structure/designs/{rule}.yaml"));
        let expected: Vec<(&str, &str)> = subjects.iter().map(|s| (*rule, *s)).collect();
        assert_eq!(violated(&report), expected, "{:#?}", report.violations);
        assert!(!report.valid, "{rule}");
        assert_eq!(report.exit_code(), 1, "{rule}");
        assert!(report.violations.iter().all(|v| v.gate), "{rule}");
        for (id, count) in &report.kpis {
            let expected = if id == rule { subjects.len() } else { 0 };
            assert_eq!(*count, expected, "{rule}: kpi {id}");
        }
    }
}

#[test]
fn the_fixture_directory_holds_one_design_per_rule_and_the_valid_one() {
    let files = yaml_files("structure/designs");
    assert_eq!(files.len(), structure::RULES.len() + 1, "{files:#?}");
}

#[test]
fn a_delegation_passes_on_at_most_what_its_source_holds() {
    let report = design("structure/designs/delegation_exceeds_limit.yaml");
    let message = &report.violations[0].message;
    assert!(
        message.contains("limit 500 is above the limit 100"),
        "{message}"
    );
}

#[test]
fn an_escalation_ends_at_a_part_that_holds_a_class_and_closes_no_cycle() {
    let report = design("structure/designs/escalation_dead_end.yaml");
    let messages: Vec<&str> = report
        .violations
        .iter()
        .map(|v| v.message.as_str())
        .collect();
    assert!(
        messages[0].contains("closes a cycle through board, third"),
        "{messages:?}"
    );
    assert!(
        messages[1].contains("holds no decision class"),
        "{messages:?}"
    );
}
