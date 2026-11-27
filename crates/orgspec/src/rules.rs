//! Selectors, the defaults and the rule forms of the types, applied to a design.

use crate::blocks;
use crate::check::Violation;
use crate::design::{Link, Model, Part};
use crate::graph;
use crate::node::Node;
use crate::types::{Direction, Form, Same, Scope, Selector};
use std::collections::{BTreeMap, BTreeSet};

/// Whether the selector picks the part: every term matches.
pub fn matches(m: &Model, selector: &Selector, part: &Part) -> bool {
    let of_kind = |part: &Part, wanted: &[Node]| {
        m.kinds(part)
            .iter()
            .any(|kind| wanted.iter().any(|w| w.text() == kind.name))
    };
    selector
        .terms
        .iter()
        .all(|(key, wanted)| match key.as_str() {
            "kind" => of_kind(part, wanted),
            "role_kind" => m
                .text(part, "role")
                .and_then(|role| m.part(&role))
                .is_some_and(|role| role.block() == Some("role") && of_kind(role, wanted)),
            field => m.value(part, field).is_some_and(|value| match &value {
                Node::Seq(items) => items.iter().any(|i| wanted.iter().any(|w| w.same(i))),
                one => wanted.iter().any(|w| w.same(one)),
            }),
        })
}

/// Whether `other` lies where `same` wants it, seen from `part`.
fn placed(m: &Model, same: Option<Same>, part: &Part, other: &Part) -> bool {
    let units = || m.unit_of(part).zip(m.unit_of(other));
    match same {
        None => true,
        Some(Same::Unit) => units().is_some_and(|(a, b)| a == b),
        Some(Same::Parent) => units().is_some_and(|(a, b)| same_parent(m, &a, &b)),
    }
}

/// Two units are equal or are `in` the same part.
fn same_parent(m: &Model, a: &str, b: &str) -> bool {
    a == b
        || m.parent_of(a)
            .zip(m.parent_of(b))
            .is_some_and(|(x, y)| x == y)
}

/// Applies every default once, in the order of their ids, and returns the links each added.
pub fn apply_defaults(m: &mut Model) -> BTreeMap<String, Vec<String>> {
    let (design, set) = (m.design, m.set);
    let mut added = BTreeMap::new();
    for default in set.defaults() {
        let mut links = Vec::new();
        for part in &design.parts {
            let carries_already = m.links.iter().any(|link| {
                link.from.as_deref() == Some(part.name.as_str())
                    && link.carries.as_deref() == Some(default.carries.as_str())
            });
            if carries_already || !matches(m, &default.each, part) {
                continue;
            }
            let targets: Vec<&Part> = design
                .parts
                .iter()
                .filter(|other| {
                    other.name != part.name
                        && matches(m, &default.to, other)
                        && placed(m, default.to.same, part, other)
                })
                .collect();
            if let [target] = targets.as_slice() {
                links.push(Link::added(
                    &part.name,
                    &target.name,
                    &default.carries,
                    &default.id,
                ));
            }
        }
        added.insert(default.id.clone(), links.iter().map(Link::name).collect());
        m.links.extend(links);
    }
    added
}

fn count_fault(n: u64, min: Option<u64>, max: Option<u64>) -> Option<String> {
    match (min, max) {
        (Some(min), _) if n < min => Some(format!("fewer than min {min}")),
        (_, Some(max)) if n > max => Some(format!("more than max {max}")),
        _ => None,
    }
}

/// Applies every rule of the design's types and returns the violations.
pub fn check(m: &Model) -> Vec<Violation> {
    let mut out = Vec::new();
    let parts = &m.design.parts;
    for (_, rule, gate) in m.set.rules() {
        let mut add = |subject: &str, message: String| {
            out.push(Violation {
                rule: rule.id.clone(),
                gate,
                subject: subject.to_string(),
                message,
            });
        };
        let carrying = |carries: &[String]| -> Vec<(&Part, &Part, &Link)> {
            m.links
                .iter()
                .filter(|link| link.carries.as_ref().is_some_and(|c| carries.contains(c)))
                .filter_map(|link| {
                    let from = m.part(link.from.as_deref()?)?;
                    Some((from, m.part(link.to.as_deref()?)?, link))
                })
                .collect()
        };
        match &rule.form {
            Form::Restate | Form::Unknown => {}
            Form::Parts {
                matching,
                per,
                min,
                max,
            } => {
                let picked: Vec<&Part> = parts.iter().filter(|p| matches(m, matching, p)).collect();
                let Some(per) = per else {
                    let n = picked.len() as u64;
                    if let Some(fault) = count_fault(n, *min, *max) {
                        let message =
                            format!("has {n} part(s) matching {}, {fault}", matching.text());
                        add(&m.design.organization, message);
                    }
                    continue;
                };
                for holder in parts.iter().filter(|p| matches(m, per, p)) {
                    let inside = |p: &&&Part| {
                        p.name != holder.name
                            && blocks::PLACING
                                .iter()
                                .any(|f| m.text(p, f).as_deref() == Some(holder.name.as_str()))
                    };
                    let n = picked.iter().filter(inside).count() as u64;
                    if let Some(fault) = count_fault(n, *min, *max) {
                        let message =
                            format!("has {n} part(s) matching {}, {fault}", matching.text());
                        add(&holder.name, message);
                    }
                }
            }
            Form::Links {
                each,
                carries,
                direction,
                other,
                min,
                max,
            } => {
                let links = carrying(carries);
                for part in parts.iter().filter(|p| matches(m, each, p)) {
                    let n = links
                        .iter()
                        .filter(|(from, to, _)| {
                            let (this, that) = match direction {
                                Direction::In => (to, from),
                                Direction::Out => (from, to),
                            };
                            this.name == part.name
                                && other.as_ref().is_none_or(|o| matches(m, o, that))
                        })
                        .count() as u64;
                    if let Some(fault) = count_fault(n, *min, *max) {
                        let way = match direction {
                            Direction::In => "incoming",
                            Direction::Out => "outgoing",
                        };
                        let message = format!(
                            "has {n} {way} link(s) carrying {}, {fault}",
                            carries.join(" or ")
                        );
                        add(&part.name, message);
                    }
                }
            }
            Form::Forbid { carries, from, to } => {
                for (source, target, link) in carrying(carries) {
                    if matches(m, from, source) && matches(m, to, target) {
                        let message = format!(
                            "no link carrying {} may run from {} to {}",
                            carries.join(" or "),
                            from.text(),
                            to.text()
                        );
                        add(&link.name(), message);
                    }
                }
            }
            Form::Separate { carries, to } => {
                let mut carried: BTreeMap<(&str, &str), BTreeSet<&str>> = BTreeMap::new();
                for (source, target, link) in carrying(carries) {
                    if to.as_ref().is_none_or(|to| matches(m, to, target)) {
                        carried
                            .entry((source.name.as_str(), target.name.as_str()))
                            .or_default()
                            .extend(link.carries.as_deref());
                    }
                }
                for ((source, target), names) in carried.iter().filter(|(_, n)| n.len() > 1) {
                    let names: Vec<&str> = names.iter().copied().collect();
                    let message = format!(
                        "carries {} to `{target}`; one part carries one of them to one target",
                        names.join(" and ")
                    );
                    add(source, message);
                }
            }
            Form::Within { carries, scope } => {
                for (source, target, link) in carrying(carries) {
                    let units = || m.unit_of(source).zip(m.unit_of(target));
                    let (together, word) = match scope {
                        Scope::Unit => (units().is_some_and(|(a, b)| a == b), "unit"),
                        Scope::Parent => (
                            units().is_some_and(|(a, b)| same_parent(m, &a, &b)),
                            "unit or under the same parent",
                        ),
                        Scope::Organization => (
                            m.organization_of(source)
                                .zip(m.organization_of(target))
                                .is_some_and(|(a, b)| a == b),
                            "organization",
                        ),
                    };
                    if !together {
                        add(
                            &link.name(),
                            format!("its two ends are not in the same {word}"),
                        );
                    }
                }
            }
            Form::Span { each, max, seats } => {
                let held = |p: &&Part| {
                    p.block() == Some("position") && each.as_ref().is_none_or(|e| matches(m, e, p))
                };
                for part in parts.iter().filter(held) {
                    let reporting = m.reporting_to(&part.name);
                    let (n, what) = if *seats {
                        (
                            reporting.iter().map(|p| m.seats(p)).sum::<i64>() as u64,
                            "seat(s)",
                        )
                    } else {
                        (reporting.len() as u64, "position(s)")
                    };
                    if n > *max {
                        add(
                            &part.name,
                            format!("{n} {what} report to it, more than max {max}"),
                        );
                    }
                }
            }
            Form::Depth { max } => {
                let reports = m.graph("reports_to");
                let reported_to: BTreeSet<&String> = reports.values().flatten().collect();
                for name in reports.keys().filter(|name| !reported_to.contains(name)) {
                    let depth = graph::longest(&reports, name) as u64;
                    if depth > *max {
                        let message = format!(
                            "its chain of `reports_to` has {depth} links, more than max {max}"
                        );
                        add(name, message);
                    }
                }
            }
            Form::Value {
                each,
                field,
                limits,
                required,
            } => {
                for part in parts.iter().filter(|p| matches(m, each, p)) {
                    match m.value(part, field) {
                        None if *required => add(&part.name, format!("states no `{field}`")),
                        None => {}
                        Some(value) => {
                            if let Some(fault) = limits.fault(&value) {
                                let message = format!("`{field}: {}` {fault}", value.text());
                                add(&part.name, message);
                            }
                        }
                    }
                }
            }
        }
    }
    out
}
