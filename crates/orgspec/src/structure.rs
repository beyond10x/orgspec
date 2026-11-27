//! The structural rules of every design, whatever its type, and the checks of each field.
//!
//! Every rule here is a gate.

use crate::blocks::{self, Block, Field, Ty};
use crate::check::Violation;
use crate::design::{Link, Model, Part};
use crate::graph;
use crate::node::Node;
use std::collections::{BTreeMap, BTreeSet};

/// The id of every structural rule: the rules of `design/building-blocks.md`, then the checks
/// of fields.
pub const RULES: &[&str] = &[
    "unknown_block",
    "unknown_carries",
    "unknown_link_parts",
    "part_outside_organization",
    "unknown_role",
    "reporting_cycle",
    "subject_without_accountable",
    "subject_with_two_accountables",
    "accountable_not_position",
    "agent_without_accountable_human",
    "class_without_decider",
    "class_with_two_deciders",
    "decider_not_allowed",
    "delegation_without_right",
    "delegation_exceeds_limit",
    "demand_without_offering",
    "escalation_dead_end",
    "border_without_part",
    "unreachable_parts",
    "unknown_attribute",
    "missing_attribute",
    "bad_value",
    "unknown_kind",
    "kind_block_mismatch",
    "kind_dimension_mismatch",
    "unknown_reference",
];

struct Out(Vec<Violation>);

impl Out {
    fn add(&mut self, rule: &'static str, subject: &str, message: impl Into<String>) {
        debug_assert!(RULES.contains(&rule), "{rule}");
        self.0.push(Violation {
            rule: rule.to_string(),
            gate: true,
            subject: subject.to_string(),
            message: message.into(),
        });
    }
}

/// Applies every structural rule to the design and the links its defaults added.
pub fn check(m: &Model) -> Vec<Violation> {
    let mut out = Out(Vec::new());
    for key in &m.design.unknown_keys {
        out.add(
            "unknown_attribute",
            key,
            "a design document has no such key",
        );
    }
    for part in &m.design.parts {
        check_part(m, part, &mut out);
    }
    let carried = m.set.carried();
    for link in m.links.iter().filter(|link| link.added_by.is_none()) {
        check_link(m, link, &carried, &mut out);
    }
    reporting(m, &mut out);
    accountability(m, &mut out);
    agents(m, &mut out);
    decisions(m, &mut out);
    delegations(m, &mut out);
    demand(m, &mut out);
    escalations(m, &mut out);
    border(m, &mut out);
    unreachable(m, &mut out);
    out.0
}

/// The rule that reports a reference that names no part of an allowed block.
fn reference_rule(block: &str, field: &str) -> &'static str {
    match (block, field) {
        ("unit" | "position", "in") => "part_outside_organization",
        ("position", "role") => "unknown_role",
        _ => "unknown_reference",
    }
}

fn names_part(m: &Model, name: &str, allowed: &[&str]) -> bool {
    m.part(name)
        .is_some_and(|p| allowed.is_empty() || p.block().is_some_and(|b| allowed.contains(&b)))
}

fn wanted(allowed: &[&str]) -> String {
    if allowed.is_empty() {
        "part".to_string()
    } else {
        format!("{} part", allowed.join(" or "))
    }
}

fn check_part(m: &Model, part: &Part, out: &mut Out) {
    let name = part.name.as_str();
    if let Some(fault) = &part.vary_fault {
        out.add("bad_value", name, fault.clone());
    }
    let Some(block) = part.block().and_then(blocks::block) else {
        let message = match part.get("block") {
            None => "names no block".to_string(),
            Some(named) => format!("`{}` is no block a part may name", named.text()),
        };
        out.add("unknown_block", name, message);
        return;
    };
    let mut kind_known = true;
    if let Some(kind) = part.get("kind") {
        match kind.as_str().and_then(|k| m.set.kind(k)) {
            None => {
                kind_known = false;
                let message = format!("`{}` is no kind of the design's types", kind.text());
                out.add("unknown_kind", name, message);
            }
            Some(known) if known.block != block.name => {
                kind_known = false;
                let message = format!(
                    "is a `{}` and `{}` is a kind of `{}`",
                    block.name, known.name, known.block
                );
                out.add("kind_block_mismatch", name, message);
            }
            Some(_) => {}
        }
    }
    let kinds = m.kinds(part);
    for (key, value) in &part.fields {
        if key == "block" || key == "kind" {
            continue;
        }
        if key == "summary" || key == "title" {
            if value.as_str().is_none() {
                let message = format!("`{key}: {}` is not a string", value.text());
                out.add("bad_value", name, message);
            }
        } else if let Some(field) = block.field(key) {
            check_field(m, part, block, field, value, out);
        } else if let Some(attribute) = kinds.iter().find_map(|k| k.attribute(key)) {
            if let Some(fault) = attribute.ty.fault(value) {
                out.add(
                    "bad_value",
                    name,
                    format!("`{key}: {}` {fault}", value.text()),
                );
            }
        } else if kind_known {
            let message = match kinds.first() {
                Some(kind) => format!(
                    "block `{}` has no field `{key}` and kind `{}` no such attribute",
                    block.name, kind.name
                ),
                None => format!("block `{}` has no field `{key}`", block.name),
            };
            out.add("unknown_attribute", name, message);
        }
    }
    for field in block.fields.iter().filter(|f| f.required) {
        if m.value(part, field.name).is_none() {
            let rule = reference_rule(block.name, field.name);
            out.add(rule, name, format!("states no `{}`", field.name));
        }
    }
    for kind in &kinds {
        for attribute in kind.attributes.iter().filter(|a| a.required) {
            if part.get(&attribute.name).is_none() {
                let message = format!("kind `{}` requires `{}`", kind.name, attribute.name);
                out.add("missing_attribute", name, message);
            }
        }
        for (field, fixed) in &kind.fixed {
            if let Some(stated) = part.get(field)
                && !stated.same(fixed)
            {
                let message = format!(
                    "states `{field}: {}` and kind `{}` fixes `{field}: {}`",
                    stated.text(),
                    kind.name,
                    fixed.text()
                );
                out.add("kind_dimension_mismatch", name, message);
            }
        }
    }
}

fn check_field(m: &Model, part: &Part, block: &Block, field: &Field, value: &Node, out: &mut Out) {
    let name = part.name.as_str();
    if let Some(fault) = field.ty.fault(value) {
        let message = format!("`{}: {}` {fault}", field.name, value.text());
        out.add("bad_value", name, message);
        return;
    }
    match (&field.ty, value) {
        (Ty::Ref(allowed), target) => {
            let target = target.text();
            if !names_part(m, &target, allowed) {
                let message = format!(
                    "`{}` names `{target}`, which is no {}",
                    field.name,
                    wanted(allowed)
                );
                out.add(reference_rule(block.name, field.name), name, message);
            }
        }
        (Ty::Steps, Node::Seq(steps)) => {
            for (i, step) in steps.iter().enumerate() {
                check_step(m, name, i + 1, step, out);
            }
        }
        _ => {}
    }
}

fn check_step(m: &Model, part: &str, number: usize, step: &Node, out: &mut Out) {
    for (key, value) in step.entries() {
        let shown = format!("step {number}: `{key}: {}`", value.text());
        match key.as_str() {
            "name" => {
                if let Some(fault) = Ty::Text.fault(value) {
                    out.add("bad_value", part, format!("{shown} {fault}"));
                }
            }
            "activity" => {
                if let Some(fault) = Ty::Enum(blocks::ACTIVITIES).fault(value) {
                    out.add("bad_value", part, format!("{shown} {fault}"));
                }
            }
            "role" => match value.scalar_text() {
                None => out.add(
                    "bad_value",
                    part,
                    format!("{shown} is not the name of a part"),
                ),
                Some(role) if !m.is(&role, "role") => {
                    let message = format!("step {number} names `{role}`, which is no role part");
                    out.add("unknown_reference", part, message);
                }
                Some(_) => {}
            },
            _ => {
                debug_assert!(!blocks::STEP_KEYS.contains(&key.as_str()));
                let message = format!("step {number}: a step has no key `{key}`");
                out.add("unknown_attribute", part, message);
            }
        }
    }
}

fn check_link(m: &Model, link: &Link, carried: &BTreeSet<&str>, out: &mut Out) {
    let name = link.name();
    if let Some(fault) = &link.vary_fault {
        out.add("bad_value", &name, fault.clone());
    }
    for (key, value) in &link.fields {
        let fault = match key.as_str() {
            "limit" => Ty::Number.fault(value),
            "mode" => Ty::Enum(blocks::MODES).fault(value),
            "after" => Ty::Duration.fault(value),
            "scope" | "class" | "via" => Ty::Ref(&[]).fault(value),
            known if blocks::LINK_KEYS.contains(&known) => None,
            _ => {
                out.add(
                    "unknown_attribute",
                    &name,
                    format!("a link has no key `{key}`"),
                );
                None
            }
        };
        if let Some(fault) = fault {
            out.add(
                "bad_value",
                &name,
                format!("`{key}: {}` {fault}", value.text()),
            );
        }
    }
    for (key, block) in [
        ("scope", "unit"),
        ("class", "decision_class"),
        ("via", "channel"),
    ] {
        if let Some(target) = link.text(key)
            && !m.is(&target, block)
        {
            let message = format!("`{key}` names `{target}`, which is no {block} part");
            out.add("unknown_reference", &name, message);
        }
    }
    match &link.carries {
        None => out.add("unknown_carries", &name, "says nothing it carries"),
        Some(carries) if !carried.contains(carries.as_str()) => {
            let message =
                format!("`{carries}` is carried neither by the blocks nor by the design's types");
            out.add("unknown_carries", &name, message);
        }
        Some(_) => {}
    }
    for (end, named) in [("from", &link.from), ("to", &link.to)] {
        match named {
            None => out.add("unknown_link_parts", &name, format!("names no `{end}`")),
            Some(part) if m.part(part).is_none() => {
                let message = format!("`{end}` names `{part}`, which is no part");
                out.add("unknown_link_parts", &name, message);
            }
            Some(_) => {}
        }
    }
}

fn reporting(m: &Model, out: &mut Out) {
    for group in graph::cycles(&m.graph("reports_to")) {
        let message = format!("`reports_to` closes a cycle through {}", group.join(", "));
        out.add("reporting_cycle", &group[0], message);
    }
}

fn accountability(m: &Model, out: &mut Out) {
    for (from, _, link) in m.carrying("accountable") {
        if !m.is(from, "position") {
            let message = format!("`{from}` is no position");
            out.add("accountable_not_position", &link.name(), message);
        }
    }
    for part in &m.design.parts {
        if !part.block().is_some_and(|b| blocks::SUBJECTS.contains(&b)) {
            continue;
        }
        let accountable: Vec<&str> = m
            .carrying("accountable")
            .filter(|(_, to, _)| *to == part.name)
            .map(|(from, _, _)| from)
            .collect();
        match accountable.len() {
            0 => out.add(
                "subject_without_accountable",
                &part.name,
                "no `accountable` link ends here",
            ),
            1 => {}
            n => {
                let message = format!(
                    "{n} `accountable` links end here: from {}",
                    accountable.join(", ")
                );
                out.add("subject_with_two_accountables", &part.name, message);
            }
        }
    }
}

fn agents(m: &Model, out: &mut Out) {
    let answers = m.graph("answers_to");
    let held_by = |part: &Part| {
        m.text(part, "held_by")
            .filter(|_| part.block() == Some("position"))
    };
    for part in &m.design.parts {
        if !matches!(held_by(part).as_deref(), Some("agent" | "mixed")) {
            continue;
        }
        let human = graph::reach(&answers, &part.name).iter().any(|reached| {
            m.part(reached)
                .is_some_and(|p| held_by(p).as_deref() == Some("human"))
        });
        if !human {
            let message = "no chain of `answers_to` links ends at a position held by humans";
            out.add("agent_without_accountable_human", &part.name, message);
        }
    }
}

fn decisions(m: &Model, out: &mut Out) {
    for part in &m.design.parts {
        if part.block() != Some("decision_class") {
            continue;
        }
        let mut by_scope: BTreeMap<Option<String>, Vec<&str>> = BTreeMap::new();
        for (from, _, link) in m.carrying("decides").filter(|(_, to, _)| *to == part.name) {
            by_scope.entry(link.text("scope")).or_default().push(from);
        }
        if by_scope.is_empty() {
            out.add(
                "class_without_decider",
                &part.name,
                "no `decides` link ends here",
            );
        }
        for (scope, deciders) in by_scope.iter().filter(|(_, d)| d.len() > 1) {
            let scope = match scope {
                Some(unit) => format!("the scope `{unit}`"),
                None => "the whole organization".to_string(),
            };
            let message = format!(
                "{} `decides` links for {scope}: from {}",
                deciders.len(),
                deciders.join(", ")
            );
            out.add("class_with_two_deciders", &part.name, message);
        }
    }
    for (from, _, link) in m.carrying("decides") {
        let allowed = m.part(from).is_some_and(|p| match p.block() {
            Some("position") => true,
            Some("unit") => m.text(p, "nature").as_deref() == Some("governing"),
            _ => false,
        });
        if !allowed {
            let message = format!("`{from}` is neither a position nor a governing unit");
            out.add("decider_not_allowed", &link.name(), message);
        }
    }
}

fn delegations(m: &Model, out: &mut Out) {
    let held = m.holdings();
    for (from, _, link) in m.carrying("delegates") {
        let Some(class) = link.text("class") else {
            out.add("delegation_without_right", &link.name(), "names no `class`");
            continue;
        };
        if !m.is(&class, "decision_class") {
            continue;
        }
        match held.get(&(from.to_string(), class.clone())) {
            None => {
                let message =
                    format!("`{from}` neither decides `{class}` nor holds it by a delegation");
                out.add("delegation_without_right", &link.name(), message);
            }
            Some(&has) => {
                if let Some(limit) = link.limit().filter(|limit| *limit > has) {
                    let message = format!(
                        "limit {} is above the limit {} that `{from}` holds for `{class}`",
                        Node::Float(limit).text(),
                        Node::Float(has).text()
                    );
                    out.add("delegation_exceeds_limit", &link.name(), message);
                }
            }
        }
    }
}

fn demand(m: &Model, out: &mut Out) {
    for (from, to, link) in m.carrying("demand") {
        if !(m.is(from, "relationship") && m.is(to, "offering")) {
            let message = "does not run from a relationship to an offering";
            out.add("demand_without_offering", &link.name(), message);
        }
    }
}

fn escalations(m: &Model, out: &mut Out) {
    let escalates = m.graph("escalates");
    let holders: BTreeSet<String> = m.holdings().into_keys().map(|(part, _)| part).collect();
    for group in graph::cycles(&escalates) {
        let message = format!("`escalates` closes a cycle through {}", group.join(", "));
        out.add("escalation_dead_end", &group[0], message);
    }
    for (name, next) in &escalates {
        if next.is_empty() && !holders.contains(name) {
            let message = "escalations end here and it holds no decision class";
            out.add("escalation_dead_end", name, message);
        }
    }
}

fn border(m: &Model, out: &mut Out) {
    for entry in &m.design.border {
        if m.part(entry).is_none() {
            out.add(
                "border_without_part",
                entry,
                "the border names it and it is no part",
            );
        }
    }
}

fn unreachable(m: &Model, out: &mut Out) {
    let mut named: BTreeSet<String> = m.design.border.iter().cloned().collect();
    for link in &m.links {
        named.extend(
            ["from", "to", "scope", "class", "via"]
                .iter()
                .filter_map(|key| link.text(key)),
        );
    }
    for part in &m.design.parts {
        let Some(block) = part.block().and_then(blocks::block) else {
            continue;
        };
        for (key, value) in &part.fields {
            match (block.field(key).map(|f| &f.ty), value) {
                (Some(Ty::Ref(_)), target) => {
                    named.extend(target.scalar_text().filter(|t| *t != part.name));
                }
                (Some(Ty::Steps), Node::Seq(steps)) => {
                    named.extend(
                        steps
                            .iter()
                            .filter_map(|s| s.get("role").and_then(Node::scalar_text)),
                    );
                }
                _ => {}
            }
        }
    }
    for part in &m.design.parts {
        if part.block() != Some("organization") && !named.contains(&part.name) {
            let message = "no link, no field of another part and no border entry names it";
            out.add("unreachable_parts", &part.name, message);
        }
    }
}
