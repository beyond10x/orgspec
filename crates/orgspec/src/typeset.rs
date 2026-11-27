//! The type path, the set of types a name resolves to, and what a set is refused for.

use crate::blocks;
use crate::graph;
use crate::node::{self, Node};
use crate::structure;
use crate::types::{
    self, DefaultRule, Form, Kind, Limits, Refusal, Rule, Selector, TypeDoc, refusal, selector_text,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

/// Every reason `orgspec type check` refuses a type for.
pub const REFUSALS: &[&str] = &[
    "unknown_type",
    "type_cycle",
    "type_declared_twice",
    "kind_declared_twice",
    "unknown_block",
    "kind_unknown_dimension",
    "kind_dimension_conflict",
    "kind_is_unknown",
    "kind_is_other_block",
    "kind_is_cycle",
    "attribute_shadows_dimension",
    "link_declared_twice",
    "rule_declared_twice",
    "unknown_rule_form",
    "unknown_key",
    "unknown_carries",
    "unknown_kind",
    "gate_loosened",
    "bounds_unsatisfiable",
    "kpi_over_party",
];

/// The directories type documents are read from, in order: each `--type-path`, each entry of
/// `ORGSPEC_TYPE_PATH`, `types` in the working directory if it exists, and `beside` if it exists.
pub fn type_dirs(
    flags: &[PathBuf],
    env: Option<&str>,
    cwd: &Path,
    beside: Option<&Path>,
) -> Vec<PathBuf> {
    let mut dirs = flags.to_vec();
    dirs.extend(
        env.unwrap_or("")
            .split(':')
            .filter(|entry| !entry.is_empty())
            .map(PathBuf::from),
    );
    for dir in [Some(cwd.join("types")), beside.map(Path::to_path_buf)]
        .into_iter()
        .flatten()
    {
        if dir.is_dir() {
            dirs.push(dir);
        }
    }
    dirs
}

/// The directory of a file.
pub fn directory(file: &Path) -> PathBuf {
    match file.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

struct Entry {
    name: String,
    file: PathBuf,
    real: PathBuf,
    doc: Node,
}

/// Every type document found on the type path.
#[derive(Default)]
pub struct TypePath {
    entries: Vec<Entry>,
    /// The `*.yaml` files that could not be read as YAML.
    unread: Vec<String>,
}

/// The types a list of names resolves to, or why they are refused.
#[derive(Debug, Default)]
pub struct Resolution {
    pub set: TypeSet,
    pub refusals: Vec<Refusal>,
}

impl TypePath {
    /// Reads every `*.yaml` with the type format in `dirs`. A directory named twice is read once.
    pub fn scan(dirs: &[PathBuf]) -> Result<TypePath, String> {
        let mut path = TypePath::default();
        let mut seen = BTreeSet::new();
        for dir in dirs {
            let unreadable = |e: std::io::Error| format!("type path {}: {e}", dir.display());
            if !seen.insert(std::fs::canonicalize(dir).map_err(unreadable)?) {
                continue;
            }
            let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
                .map_err(unreadable)?
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|file| file.extension().is_some_and(|x| x == "yaml") && file.is_file())
                .collect();
            files.sort();
            for file in files {
                path.add(&file)?;
            }
        }
        Ok(path)
    }

    /// Adds one file unless it is on the path already or is no type document.
    pub fn add(&mut self, file: &Path) -> Result<(), String> {
        let real = std::fs::canonicalize(file).unwrap_or_else(|_| file.to_path_buf());
        if self.entries.iter().any(|e| e.real == real) {
            return Ok(());
        }
        let doc = match node::load(file) {
            Ok(doc) => doc,
            Err(e) => {
                self.unread.push(format!("{}: {e}", file.display()));
                return Ok(());
            }
        };
        if doc.get("format").and_then(Node::as_str) != Some(types::FORMAT) {
            return Ok(());
        }
        let name = doc
            .get("type")
            .and_then(Node::as_str)
            .ok_or_else(|| format!("{}: missing key `type`", file.display()))?
            .to_string();
        self.entries.push(Entry {
            name,
            file: file.to_path_buf(),
            real,
            doc,
        });
        Ok(())
    }

    /// Resolves `roots` and everything they extend. `named_by` says who names the roots.
    /// Fails when a document of the set does not have the shape of the format.
    pub fn resolve(&self, roots: &[String], named_by: &str) -> Result<Resolution, String> {
        let mut refusals = Vec::new();
        for entry in self.entries.iter().filter(|e| e.name == "base") {
            refusals.push(refusal(
                "type_declared_twice",
                "base",
                format!(
                    "`base` is built in and {} declares it",
                    entry.file.display()
                ),
            ));
        }
        let mut types: BTreeMap<String, TypeDoc> = BTreeMap::new();
        let mut refused: BTreeSet<String> = BTreeSet::new();
        let mut queue: VecDeque<(String, String)> = roots
            .iter()
            .map(|root| (root.clone(), named_by.to_string()))
            .collect();
        while let Some((name, by)) = queue.pop_front() {
            if name == "base" || types.contains_key(&name) || !refused.insert(name.clone()) {
                continue;
            }
            let found: Vec<&Entry> = self.entries.iter().filter(|e| e.name == name).collect();
            match found.as_slice() {
                [] => {
                    let mut message =
                        format!("{by} names it and no document on the type path declares it");
                    if !self.unread.is_empty() {
                        message += &format!(" (not read: {})", self.unread.join("; "));
                    }
                    refusals.push(refusal("unknown_type", &name, message));
                }
                [one] => {
                    let read = types::read(&one.doc, &name, &one.file)
                        .map_err(|e| format!("{}: {e}", one.file.display()))?;
                    let by = format!("type `{name}`");
                    queue.extend(read.extends.iter().map(|x| (x.clone(), by.clone())));
                    refused.remove(&name);
                    types.insert(name, read);
                }
                many => {
                    let files: Vec<String> =
                        many.iter().map(|e| e.file.display().to_string()).collect();
                    refusals.push(refusal(
                        "type_declared_twice",
                        &name,
                        format!("declared by {}", files.join(" and ")),
                    ));
                }
            }
        }
        let extends = |doc: &TypeDoc| -> Vec<String> {
            doc.extends
                .iter()
                .filter(|x| types.contains_key(*x))
                .cloned()
                .collect()
        };
        let edges: graph::Graph = types
            .iter()
            .map(|(name, doc)| (name.clone(), extends(doc)))
            .collect();
        for group in graph::cycles(&edges) {
            refusals.push(refusal(
                "type_cycle",
                &group[0],
                format!("extends itself through {}", group.join(", ")),
            ));
        }
        if !refusals.is_empty() {
            refusals.sort();
            return Ok(Resolution {
                set: TypeSet::default(),
                refusals,
            });
        }
        let mut set = TypeSet::default();
        let mut placed: BTreeSet<String> = BTreeSet::new();
        while placed.len() < types.len() {
            let ready: Vec<String> = types
                .iter()
                .filter(|(name, doc)| {
                    !placed.contains(*name) && extends(doc).iter().all(|x| placed.contains(x))
                })
                .map(|(name, _)| name.clone())
                .collect();
            for name in ready {
                set.types.push(types[&name].clone());
                placed.insert(name);
            }
        }
        let refusals = set.refusals();
        Ok(Resolution { set, refusals })
    }
}

/// A set of types: some types and everything they extend, the extended ones first. `base` is
/// in every set and has nothing to hold.
#[derive(Debug, Default)]
pub struct TypeSet {
    pub types: Vec<TypeDoc>,
}

/// Notes the id of a rule or default of one type, or refuses it when the id is taken.
fn declare<'a>(
    id: &'a str,
    subject: &str,
    declared: &BTreeMap<&str, String>,
    ids: &mut Vec<&'a str>,
    out: &mut Vec<Refusal>,
) {
    if let Some(by) = declared.get(id) {
        let message = format!("{by} already have a rule or default of that id");
        out.push(refusal("rule_declared_twice", subject, message));
    } else if ids.contains(&id) {
        let message = "the type declares the id twice";
        out.push(refusal("rule_declared_twice", subject, message));
    } else {
        ids.push(id);
    }
}

fn range(min: u64, max: u64) -> String {
    if max == u64::MAX {
        format!("{min}..")
    } else {
        format!("{min}..{max}")
    }
}

impl TypeSet {
    /// The names of the set, `base` first.
    pub fn names(&self) -> Vec<String> {
        std::iter::once("base".to_string())
            .chain(self.types.iter().map(|t| t.name.clone()))
            .collect()
    }

    pub fn kinds(&self) -> impl Iterator<Item = (&TypeDoc, &Kind)> {
        self.types
            .iter()
            .flat_map(|t| t.kinds.iter().map(move |k| (t, k)))
    }

    pub fn kind(&self, name: &str) -> Option<&Kind> {
        self.kinds().map(|(_, k)| k).find(|k| k.name == name)
    }

    /// The kind and every kind it refines, the narrowest first.
    pub fn kind_chain(&self, name: &str) -> Vec<&Kind> {
        let mut chain: Vec<&Kind> = Vec::new();
        let mut next = self.kind(name);
        while let Some(kind) = next {
            if chain.iter().any(|k| k.name == kind.name) {
                break;
            }
            chain.push(kind);
            next = kind.is.as_deref().and_then(|wider| self.kind(wider));
        }
        chain
    }

    /// What a link may carry: the links of the blocks and the links the types add.
    pub fn carried(&self) -> BTreeSet<&str> {
        let added = self.types.iter().flat_map(|t| &t.links).map(String::as_str);
        blocks::LINKS.iter().copied().chain(added).collect()
    }

    /// The types `name` extends, directly or through another type.
    pub fn ancestors(&self, name: &str) -> BTreeSet<String> {
        let mut found = BTreeSet::new();
        let mut stack = vec![name.to_string()];
        while let Some(at) = stack.pop() {
            for extended in self
                .types
                .iter()
                .filter(|t| t.name == at)
                .flat_map(|t| &t.extends)
            {
                if found.insert(extended.clone()) {
                    stack.push(extended.clone());
                }
            }
        }
        found
    }

    /// Every rule the set declares, with its gate after the restatements of the set.
    pub fn rules(&self) -> Vec<(&TypeDoc, &Rule, bool)> {
        let raised: BTreeSet<&str> = self
            .types
            .iter()
            .flat_map(|t| &t.rules)
            .filter(|r| matches!(r.form, Form::Restate) && r.gate == Some(true))
            .map(|r| r.id.as_str())
            .collect();
        self.types
            .iter()
            .flat_map(|t| t.rules.iter().map(move |r| (t, r)))
            .filter(|(_, r)| r.declares())
            .map(|(t, r)| {
                (
                    t,
                    r,
                    r.gate.unwrap_or(true) || raised.contains(r.id.as_str()),
                )
            })
            .collect()
    }

    /// Every default of the set, in the order of their ids.
    pub fn defaults(&self) -> Vec<&DefaultRule> {
        let mut defaults: Vec<&DefaultRule> = self.types.iter().flat_map(|t| &t.defaults).collect();
        defaults.sort_by(|a, b| a.id.cmp(&b.id));
        defaults
    }

    /// Why the set is refused; empty when it is not.
    pub fn refusals(&self) -> Vec<Refusal> {
        let mut out: Vec<Refusal> = self.types.iter().flat_map(|t| t.found.clone()).collect();
        self.check_kinds(&mut out);
        self.check_links(&mut out);
        self.check_rules(&mut out);
        self.check_gates(&mut out);
        self.check_bounds(&mut out);
        self.check_measures(&mut out);
        out.sort();
        out.dedup();
        out
    }

    fn check_kinds(&self, out: &mut Vec<Refusal>) {
        let mut declared: BTreeMap<&str, &str> = BTreeMap::new();
        for (owner, kind) in self.kinds() {
            let subject = format!("{}.{}", owner.name, kind.name);
            if let Some(other) = declared.get(kind.name.as_str()) {
                let message = if *other == owner.name {
                    "the type declares the kind twice".to_string()
                } else {
                    format!("type `{other}` declares the kind too")
                };
                out.push(refusal("kind_declared_twice", subject, message));
                continue;
            }
            declared.insert(&kind.name, &owner.name);
            let Some(block) = blocks::block(&kind.block) else {
                let message = format!("`{}` is no block a part may name", kind.block);
                out.push(refusal("unknown_block", subject, message));
                continue;
            };
            for (field, value) in &kind.fixed {
                let fault = match block.field(field) {
                    None => Some(format!("block `{}` has no field `{field}`", block.name)),
                    Some(known) if !known.ty.fixable() => {
                        Some(format!("a kind cannot fix the field `{field}`"))
                    }
                    Some(known) => known
                        .ty
                        .fault(value)
                        .map(|fault| format!("`{field}: {}` {fault}", value.text())),
                };
                if let Some(fault) = fault {
                    out.push(refusal("kind_unknown_dimension", &subject, fault));
                }
            }
            let chain = self.kind_chain(&kind.name);
            let wider = &chain[1..];
            if let Some(is) = &kind.is {
                match self.kind(is) {
                    None => out.push(refusal(
                        "kind_is_unknown",
                        &subject,
                        format!("refines `{is}`, which no type of the set declares"),
                    )),
                    Some(refined) if refined.block != kind.block => out.push(refusal(
                        "kind_is_other_block",
                        &subject,
                        format!(
                            "is a kind of `{}` and refines `{is}`, a kind of `{}`",
                            kind.block, refined.block
                        ),
                    )),
                    Some(_) => {
                        let closes = chain
                            .last()
                            .is_some_and(|last| last.is.as_deref() == Some(kind.name.as_str()));
                        if closes {
                            let names: Vec<&str> = chain.iter().map(|k| k.name.as_str()).collect();
                            out.push(refusal(
                                "kind_is_cycle",
                                &subject,
                                format!("refines itself through {}", names.join(", ")),
                            ));
                        }
                    }
                }
            }
            for attribute in &kind.attributes {
                let name = attribute.name.as_str();
                if blocks::COMMON.contains(&name) || block.field(name).is_some() {
                    out.push(refusal(
                        "attribute_shadows_dimension",
                        format!("{subject}.{name}"),
                        format!("block `{}` has a field of that name", block.name),
                    ));
                } else if let Some(other) = wider.iter().find(|k| k.attribute(name).is_some()) {
                    out.push(refusal(
                        "attribute_shadows_dimension",
                        format!("{subject}.{name}"),
                        format!(
                            "kind `{}`, which it refines, has that attribute",
                            other.name
                        ),
                    ));
                }
            }
            for (field, value) in &kind.fixed {
                let other = wider.iter().find_map(|k| {
                    k.fixes(field)
                        .filter(|fixed| !fixed.same(value))
                        .map(|fixed| (k, fixed))
                });
                if let Some((other, fixed)) = other {
                    out.push(refusal(
                        "kind_dimension_conflict",
                        &subject,
                        format!(
                            "fixes `{field}: {}` and kind `{}`, which it refines, fixes `{field}: {}`",
                            value.text(),
                            other.name,
                            fixed.text()
                        ),
                    ));
                }
            }
        }
    }

    fn check_links(&self, out: &mut Vec<Refusal>) {
        let mut declared: BTreeMap<&str, String> = blocks::LINKS
            .iter()
            .map(|link| (*link, "the blocks".to_string()))
            .collect();
        for owner in &self.types {
            for link in &owner.links {
                if let Some(by) = declared.get(link.as_str()) {
                    out.push(refusal(
                        "link_declared_twice",
                        format!("{}.{link}", owner.name),
                        format!("{by} already carry a link of that name"),
                    ));
                } else {
                    declared.insert(link, format!("type `{}`", owner.name));
                }
            }
        }
    }

    /// Whether `field` is a field or an attribute of a part the selector can pick.
    fn field_known(&self, selector: &Selector, field: &str) -> bool {
        if blocks::COMMON.contains(&field) {
            return true;
        }
        let mut named = selector.names("block");
        named.extend(
            selector
                .names("kind")
                .iter()
                .filter_map(|kind| self.kind(kind))
                .map(|kind| kind.block.clone()),
        );
        if !selector.names("role_kind").is_empty() {
            named.push("position".to_string());
        }
        let picked = |block: &str| named.is_empty() || named.iter().any(|n| n == block);
        blocks::BLOCKS
            .iter()
            .any(|b| picked(b.name) && b.field(field).is_some())
            || self
                .kinds()
                .any(|(_, k)| picked(&k.block) && k.attribute(field).is_some())
    }

    fn check_selector(&self, selector: &Selector, subject: &str, out: &mut Vec<Refusal>) {
        for block in selector.names("block") {
            if blocks::block(&block).is_none() {
                let message = format!("a selector names `{block}`, which is no block of a part");
                out.push(refusal("unknown_block", subject, message));
            }
        }
        for key in ["kind", "role_kind"] {
            for kind in selector.names(key) {
                if self.kind(&kind).is_none() {
                    let message = format!(
                        "a selector names the kind `{kind}`, which no type of the set declares"
                    );
                    out.push(refusal("unknown_kind", subject, message));
                }
            }
        }
        for (key, _) in &selector.terms {
            let special = ["block", "kind", "role_kind"].contains(&key.as_str());
            if !special && !self.field_known(selector, key) {
                let message = format!(
                    "a selector reads `{key}`, which is no selector key, no field of the parts it picks and no attribute of their kinds"
                );
                out.push(refusal("unknown_key", subject, message));
            }
        }
    }

    fn check_carries(&self, carries: &[String], subject: &str, out: &mut Vec<Refusal>) {
        let carried = self.carried();
        for name in carries {
            if !carried.contains(name.as_str()) {
                let message =
                    format!("names the link `{name}`, which neither the blocks nor a type declare");
                out.push(refusal("unknown_carries", subject, message));
            }
        }
    }

    fn check_rules(&self, out: &mut Vec<Refusal>) {
        let mut declared: BTreeMap<&str, String> = structure::RULES
            .iter()
            .map(|id| (*id, "the structural rules".to_string()))
            .collect();
        for owner in &self.types {
            let mut ids: Vec<&str> = Vec::new();
            for rule in owner.rules.iter().filter(|r| r.declares()) {
                let subject = format!("{}.{}", owner.name, rule.id);
                declare(&rule.id, &subject, &declared, &mut ids, out);
                for selector in rule.form.selectors() {
                    self.check_selector(selector, &subject, out);
                }
                self.check_carries(rule.form.carries(), &subject, out);
                if let Form::Value { each, field, .. } = &rule.form
                    && !self.field_known(each, field)
                {
                    let message = format!(
                        "reads `{field}`, which is no field of the parts it picks and no attribute of their kinds"
                    );
                    out.push(refusal("unknown_key", &subject, message));
                }
            }
            for default in &owner.defaults {
                let subject = format!("{}.{}", owner.name, default.id);
                declare(&default.id, &subject, &declared, &mut ids, out);
                self.check_selector(&default.each, &subject, out);
                self.check_selector(&default.to, &subject, out);
                self.check_carries(std::slice::from_ref(&default.carries), &subject, out);
            }
            for id in ids {
                declared.insert(id, format!("type `{}`", owner.name));
            }
        }
    }

    fn check_gates(&self, out: &mut Vec<Refusal>) {
        for owner in &self.types {
            let extended = self.ancestors(&owner.name);
            let of_extended = || {
                self.types
                    .iter()
                    .filter(|t| extended.contains(&t.name))
                    .flat_map(|t| &t.rules)
            };
            for rule in owner
                .rules
                .iter()
                .filter(|r| matches!(r.form, Form::Restate))
            {
                let subject = format!("{}.{}", owner.name, rule.id);
                let Some(restated) = of_extended().find(|r| r.declares() && r.id == rule.id) else {
                    out.push(refusal(
                        "unknown_rule_form",
                        subject,
                        "has no `rule`, and no extended type declares a rule of that id to restate",
                    ));
                    continue;
                };
                let gated = restated.gate.unwrap_or(true)
                    || of_extended().any(|r| {
                        matches!(r.form, Form::Restate) && r.id == rule.id && r.gate == Some(true)
                    });
                if gated && rule.gate == Some(false) {
                    out.push(refusal(
                        "gate_loosened",
                        subject,
                        "the rule is a gate in an extended type and `gate: false` would loosen it",
                    ));
                }
            }
        }
    }

    fn check_bounds(&self, out: &mut Vec<Refusal>) {
        struct Counted {
            key: String,
            subject: String,
            min: u64,
            max: u64,
        }
        let mut counted: Vec<Counted> = Vec::new();
        let mut valued: Vec<(String, String, &Limits)> = Vec::new();
        for (owner, rule, _) in self.rules() {
            let subject = format!("{}.{}", owner.name, rule.id);
            match &rule.form {
                Form::Parts {
                    matching,
                    per,
                    min,
                    max,
                } => counted.push(Counted {
                    key: format!(
                        "parts {} per {}",
                        matching.text(),
                        per.as_ref().map_or("-".to_string(), Selector::text)
                    ),
                    subject,
                    min: min.unwrap_or(0),
                    max: max.unwrap_or(u64::MAX),
                }),
                Form::Links {
                    each,
                    carries,
                    direction,
                    other,
                    min,
                    max,
                } => counted.push(Counted {
                    key: format!(
                        "links {} {} {direction:?} {}",
                        each.text(),
                        carries.join("|"),
                        selector_text(other.as_ref())
                    ),
                    subject,
                    min: min.unwrap_or(0),
                    max: max.unwrap_or(u64::MAX),
                }),
                Form::Value {
                    each,
                    field,
                    limits,
                    ..
                } => valued.push((format!("{} {field}", each.text()), subject, limits)),
                _ => {}
            }
        }
        for (i, later) in counted.iter().enumerate() {
            if later.min > later.max {
                let message = format!("min {} is above max {}", later.min, later.max);
                out.push(refusal("bounds_unsatisfiable", &later.subject, message));
                continue;
            }
            for earlier in &counted[..i] {
                let apart = earlier.min.max(later.min) > earlier.max.min(later.max);
                if earlier.key == later.key && earlier.min <= earlier.max && apart {
                    let message = format!(
                        "allows {} and rule `{}` allows {} of the same parts: no number is in both",
                        range(later.min, later.max),
                        earlier.subject,
                        range(earlier.min, earlier.max)
                    );
                    out.push(refusal("bounds_unsatisfiable", &later.subject, message));
                }
            }
        }
        for (i, (key, subject, limits)) in valued.iter().enumerate() {
            if !Limits::satisfiable(&[limits]) {
                let message = format!("no value is within {}", limits.text());
                out.push(refusal("bounds_unsatisfiable", subject, message));
                continue;
            }
            for (other_key, other_subject, other) in &valued[..i] {
                if other_key == key
                    && Limits::satisfiable(&[other])
                    && !Limits::satisfiable(&[limits, other])
                {
                    let message = format!(
                        "allows {} and rule `{other_subject}` allows {} for the same field: no value is in both",
                        limits.text(),
                        other.text()
                    );
                    out.push(refusal("bounds_unsatisfiable", subject, message));
                }
            }
        }
    }

    fn check_measures(&self, out: &mut Vec<Refusal>) {
        for owner in &self.types {
            for kpi in &owner.kpis {
                let subject = format!("{}.{}", owner.name, kpi.id);
                let Some(block) = &kpi.block else { continue };
                if blocks::OVER_PARTY.contains(&block.as_str()) {
                    out.push(refusal(
                        "kpi_over_party",
                        subject,
                        format!("a KPI is never read over `{block}`: nobody is scored"),
                    ));
                    continue;
                }
                if !blocks::SPECIFIED.contains(&block.as_str()) {
                    let message = format!("the KPI names `{block}`, which is no block");
                    out.push(refusal("unknown_block", subject, message));
                    continue;
                }
                if let Some(of_kind) = &kpi.of_kind {
                    let message = match self.kind(of_kind) {
                        None => format!(
                            "the KPI names the kind `{of_kind}`, which no type of the set declares"
                        ),
                        Some(kind) if &kind.block != block => format!(
                            "the KPI names the kind `{of_kind}`, which is a kind of `{}`, not of `{block}`",
                            kind.block
                        ),
                        Some(_) => continue,
                    };
                    out.push(refusal("unknown_kind", subject, message));
                }
            }
            for extension in &owner.extensions {
                if !blocks::SPECIFIED.contains(&extension.extends.as_str()) {
                    out.push(refusal(
                        "unknown_block",
                        format!("{}.{}", owner.name, extension.domain),
                        format!(
                            "the extension extends `{}`, which is no block",
                            extension.extends
                        ),
                    ));
                }
            }
        }
    }
}
