//! The two checks and their reports: a design against the structural rules and its types, and a
//! type against the types it extends.

use crate::blocks;
use crate::design::{self, Design, Link, Model};
use crate::graph;
use crate::node::{self, Node};
use crate::rules;
use crate::structure;
use crate::types::{self, Refusal};
use crate::typeset::TypePath;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// One violation of a rule by a part or a link of a design.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Violation {
    pub rule: String,
    /// Whether the rule is a gate: a gate must count 0 for the design to be valid.
    pub gate: bool,
    pub subject: String,
    pub message: String,
}

#[derive(Debug, Default, Serialize)]
pub struct Measures {
    pub positions: usize,
    pub seats: i64,
    pub agent_seats: f64,
    pub units: usize,
    pub span_max: usize,
    pub reporting_depth_max: usize,
    pub subjects: usize,
    pub subjects_with_accountable: usize,
}

/// What `orgspec check` says about one design.
#[derive(Debug, Serialize)]
pub struct DesignReport {
    pub file: String,
    pub organization: String,
    /// The types as the design names them.
    #[serde(skip)]
    pub declared: Vec<String>,
    /// The types the design is held to: what it names and everything they extend, `base` first.
    pub types: Vec<String>,
    pub parts: usize,
    /// The links the design wrote and the links its defaults added.
    pub links: usize,
    pub valid: bool,
    pub violations: Vec<Violation>,
    /// Why the design's types are refused; the design is not checked then.
    pub refusals: Vec<Refusal>,
    /// The count of every structural rule and every rule of the types.
    pub kpis: BTreeMap<String, usize>,
    /// For every default, the links it added.
    pub defaults: BTreeMap<String, Vec<String>>,
    pub measures: Measures,
    /// The names of the design's parts, in its order.
    #[serde(skip)]
    pub part_names: Vec<String>,
    /// The names of the links (`<from>-><to>:<carries>`): those the design wrote, then those
    /// its defaults added.
    #[serde(skip)]
    pub link_names: Vec<String>,
}

/// What `orgspec type check` says about one type.
#[derive(Debug, Serialize)]
pub struct TypeReport {
    pub file: String,
    #[serde(rename = "type")]
    pub name: String,
    /// The type and everything it extends, `base` first.
    pub types: Vec<String>,
    pub kinds: usize,
    pub links: usize,
    pub rules: usize,
    pub defaults: usize,
    pub kpis: usize,
    pub valid: bool,
    pub refusals: Vec<Refusal>,
}

fn refusal_lines(refusals: &[Refusal]) -> Vec<String> {
    refusals
        .iter()
        .map(|r| format!("{} {} {}", r.refusal, r.subject, r.message))
        .collect()
}

impl DesignReport {
    /// 0 when the design is valid, 1 when a gate counts a violation or a type is refused.
    pub fn exit_code(&self) -> u8 {
        u8::from(!self.valid)
    }

    /// One line per refusal or violation, then the summary line.
    pub fn lines(&self) -> Vec<String> {
        let mut lines = refusal_lines(&self.refusals);
        let head = format!(
            "orgspec check: {}: type {}",
            self.file,
            self.declared.join("+")
        );
        if !self.refusals.is_empty() {
            lines.push(format!(
                "{head} refused, {} refusal(s)",
                self.refusals.len()
            ));
            return lines;
        }
        for v in &self.violations {
            let note = if v.gate { "" } else { " (not gated)" };
            lines.push(format!("{} {} {}{note}", v.rule, v.subject, v.message));
        }
        let gated = self.violations.iter().filter(|v| v.gate);
        let gates: BTreeSet<&str> = gated.clone().map(|v| v.rule.as_str()).collect();
        lines.push(format!(
            "{head}, {} parts, {} links, {} violation(s) in {} gate(s), {} not gated",
            self.parts,
            self.links,
            gated.count(),
            gates.len(),
            self.violations.iter().filter(|v| !v.gate).count()
        ));
        lines
    }
}

impl TypeReport {
    /// 0 when the type is accepted, 1 when it is refused.
    pub fn exit_code(&self) -> u8 {
        u8::from(!self.valid)
    }

    /// One line per refusal, then the summary line.
    pub fn lines(&self) -> Vec<String> {
        let mut lines = refusal_lines(&self.refusals);
        let head = format!("orgspec type check: {}: type {}", self.file, self.name);
        lines.push(if self.valid {
            format!(
                "{head}, set {}, {} kinds, {} links, {} rules, {} defaults, {} kpis, 0 refusals",
                self.types.join("+"),
                self.kinds,
                self.links,
                self.rules,
                self.defaults,
                self.kpis
            )
        } else {
            format!("{head} refused, {} refusal(s)", self.refusals.len())
        });
        lines
    }
}

/// Reads a document and holds it to `format`.
fn load(file: &Path, format: &str) -> Result<Node, String> {
    let doc = node::load(file)?;
    match doc.get("format").map(Node::text) {
        None => Err("missing key `format`".to_string()),
        Some(found) if found != format => {
            Err(format!("unknown format `{found}`, expected `{format}`"))
        }
        Some(_) => Ok(doc),
    }
}

fn measures(m: &Model) -> Measures {
    let of = |block: &'static str| {
        m.design
            .parts
            .iter()
            .filter(move |p| p.block() == Some(block))
    };
    let reports = m.graph("reports_to");
    let accountable: BTreeSet<&str> = m.carrying("accountable").map(|(_, to, _)| to).collect();
    let subjects = || {
        m.design
            .parts
            .iter()
            .filter(|p| p.block().is_some_and(|b| blocks::SUBJECTS.contains(&b)))
    };
    Measures {
        positions: of("position").count(),
        seats: of("position").map(|p| m.seats(p)).sum(),
        agent_seats: of("position").map(|p| m.agent_seats(p)).sum(),
        units: of("unit").count(),
        span_max: of("position")
            .map(|p| m.reporting_to(&p.name).len())
            .max()
            .unwrap_or(0),
        reporting_depth_max: reports
            .keys()
            .map(|name| graph::longest(&reports, name))
            .max()
            .unwrap_or(0),
        subjects: subjects().count(),
        subjects_with_accountable: subjects()
            .filter(|p| accountable.contains(p.name.as_str()))
            .count(),
    }
}

/// Checks the design in `file` with the types found in `dirs`. Fails when the design or one of
/// its types cannot be read.
pub fn check_design(file: &Path, dirs: &[PathBuf]) -> Result<DesignReport, String> {
    let doc = load(file, design::FORMAT)?;
    let design = Design::read(&doc)?;
    let named_by = format!("the design `{}`", design.organization);
    let resolution = TypePath::scan(dirs)?.resolve(&design.types, &named_by)?;
    let mut model = Model::new(&design, &resolution.set);
    let mut report = DesignReport {
        file: file.display().to_string(),
        organization: design.organization.clone(),
        declared: design.types.clone(),
        types: resolution.set.names(),
        parts: design.parts.len(),
        links: design.links.len(),
        valid: false,
        violations: Vec::new(),
        refusals: resolution.refusals.clone(),
        kpis: BTreeMap::new(),
        defaults: BTreeMap::new(),
        measures: Measures::default(),
        part_names: design.parts.iter().map(|p| p.name.clone()).collect(),
        link_names: design.links.iter().map(Link::name).collect(),
    };
    if !report.refusals.is_empty() {
        report.measures = measures(&model);
        return Ok(report);
    }
    report.defaults = rules::apply_defaults(&mut model);
    report.links = model.links.len();
    report.link_names = model.links.iter().map(Link::name).collect();
    report.violations = structure::check(&model);
    report.violations.extend(rules::check(&model));
    report.violations.sort();
    let type_rules = resolution.set.rules();
    let ids = structure::RULES
        .iter()
        .copied()
        .chain(type_rules.iter().map(|(_, rule, _)| rule.id.as_str()));
    for id in ids {
        let count = report.violations.iter().filter(|v| v.rule == id).count();
        report.kpis.insert(id.to_string(), count);
    }
    report.valid = !report.violations.iter().any(|v| v.gate);
    report.measures = measures(&model);
    Ok(report)
}

/// Checks the type in `file` with the types found in `dirs`. Fails when the type or one it
/// extends cannot be read.
pub fn check_type(file: &Path, dirs: &[PathBuf]) -> Result<TypeReport, String> {
    let doc = load(file, types::FORMAT)?;
    let name = doc
        .get("type")
        .and_then(Node::as_str)
        .ok_or("missing key `type`")?
        .to_string();
    let mut path = TypePath::scan(dirs)?;
    path.add(file)?;
    let resolution = path.resolve(std::slice::from_ref(&name), "the command")?;
    let own = resolution.set.types.iter().find(|t| t.name == name);
    let count = |of: fn(&types::TypeDoc) -> usize| own.map_or(0, of);
    Ok(TypeReport {
        file: file.display().to_string(),
        types: resolution.set.names(),
        kinds: count(|t| t.kinds.len()),
        links: count(|t| t.links.len()),
        rules: count(|t| t.rules.len()),
        defaults: count(|t| t.defaults.len()),
        kpis: count(|t| t.kpis.len()),
        valid: resolution.refusals.is_empty(),
        refusals: resolution.refusals,
        name,
    })
}
