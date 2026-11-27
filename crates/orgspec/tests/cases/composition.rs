//! Types compose: a design of two types is held to both, a rule about a kind covers the kinds
//! that refine it, and an extending type may turn a counted rule into a gate.

use super::common::{design, violated};

#[test]
fn a_design_of_two_types_that_holds_both_is_valid() {
    let report = design("composition/designs/both-hold.yaml");
    assert!(report.valid, "{:#?}", report.violations);
    assert_eq!(report.declared, ["clinic", "agent-run"]);
    assert_eq!(report.types, ["base", "agent-run", "clinic"]);
    for rule in [
        "care_unit_has_one_lead",
        "ward_is_small",
        "agent_position_escalates",
    ] {
        assert_eq!(report.kpis.get(rule), Some(&0), "{rule}");
    }
}

#[test]
fn a_design_of_two_types_is_held_to_the_first() {
    let report = design("composition/designs/breaks-clinic.yaml");
    assert_eq!(violated(&report), [("care_unit_has_one_lead", "east")]);
    assert_eq!(report.exit_code(), 1);
}

#[test]
fn a_design_of_two_types_is_held_to_the_second() {
    let report = design("composition/designs/breaks-agent-run.yaml");
    assert_eq!(violated(&report), [("agent_position_escalates", "booking")]);
    assert_eq!(report.exit_code(), 1);
}

#[test]
fn the_order_of_the_types_does_not_matter() {
    let one = design("composition/designs/both-hold.yaml");
    let other = design("composition/designs/breaks-clinic.yaml");
    assert_eq!(other.declared, ["agent-run", "clinic"]);
    assert_eq!(one.types, other.types);
}

#[test]
fn a_rule_about_a_kind_covers_the_kind_that_refines_it() {
    // `east` is a ward; the rule is written for `care_unit`, which `ward` refines.
    let report = design("composition/designs/breaks-clinic.yaml");
    let message = &report.violations[0].message;
    assert!(message.contains("has 0 part(s) matching"), "{message}");
    assert!(message.contains("fewer than min 1"), "{message}");
}

#[test]
fn a_rule_that_is_no_gate_is_counted_and_refuses_nothing() {
    let report = design("composition/designs/big-ward.yaml");
    assert_eq!(violated(&report), [("ward_is_small", "east")]);
    assert!(!report.violations[0].gate);
    assert!(report.valid);
    assert_eq!(report.exit_code(), 0);
    assert_eq!(report.kpis["ward_is_small"], 1);
    let last = report.lines().pop().unwrap();
    assert!(
        last.ends_with("type clinic, 5 parts, 1 links, 0 violation(s) in 0 gate(s), 1 not gated"),
        "{last}"
    );
}

#[test]
fn an_extending_type_turns_the_counted_rule_into_a_gate() {
    let report = design("composition/designs/big-ward-strict.yaml");
    assert_eq!(report.types, ["base", "clinic", "strict-clinic"]);
    assert_eq!(violated(&report), [("ward_is_small", "east")]);
    assert!(report.violations[0].gate);
    assert!(!report.valid);
    assert_eq!(report.exit_code(), 1);
    let last = report.lines().pop().unwrap();
    assert!(
        last.ends_with("1 violation(s) in 1 gate(s), 0 not gated"),
        "{last}"
    );
}

#[test]
fn a_design_whose_type_is_refused_is_not_checked() {
    let report = design("refused/designs/library.yaml");
    assert!(!report.valid);
    assert_eq!(report.exit_code(), 1);
    assert!(report.violations.is_empty());
    assert_eq!(report.refusals.len(), 1);
    assert_eq!(report.refusals[0].refusal, "unknown_carries");
    assert_eq!(report.refusals[0].subject, "hasty.asset_has_curator");
    let lines = report.lines();
    assert!(
        lines[0].starts_with("unknown_carries hasty.asset_has_curator "),
        "{lines:?}"
    );
    assert!(
        lines[1].ends_with("type hasty refused, 1 refusal(s)"),
        "{lines:?}"
    );
}
