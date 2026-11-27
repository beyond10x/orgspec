//! An organization type document (`orgspec-type/1`) read into data.
//!
//! Reading fails with a message when the document does not have the shape of the format (a
//! missing key, a value of the wrong shape). What has the shape and is still refused (an unknown
//! key, an unknown rule form) is kept in [`TypeDoc::found`].

use crate::blocks;
use crate::node::{Node, duration_seconds};
use serde::Serialize;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

pub const FORMAT: &str = "orgspec-type/1";

const KEYS: &[&str] = &[
    "format",
    "type",
    "origin",
    "extends",
    "summary",
    "vocabulary",
    "kinds",
    "links",
    "rules",
    "defaults",
    "kpis",
    "extensions",
];

const RULE_KEYS: &[&str] = &["id", "summary", "gate", "rule"];
const DEFAULT_KEYS: &[&str] = &["id", "summary", "each", "carries", "to"];
const KPI_KEYS: &[&str] = &[
    "id",
    "summary",
    "block",
    "of_kind",
    "measures",
    "unit",
    "direction",
    "target",
];
const EXTENSION_KEYS: &[&str] = &["domain", "extends", "path"];
const ATTRIBUTE_KEYS: &[&str] = &["type", "required", "values", "summary"];

/// One reason a type is refused.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Refusal {
    pub refusal: &'static str,
    pub subject: String,
    pub message: String,
}

pub fn refusal(
    name: &'static str,
    subject: impl Into<String>,
    message: impl Into<String>,
) -> Refusal {
    Refusal {
        refusal: name,
        subject: subject.into(),
        message: message.into(),
    }
}

#[derive(Debug, Clone, Default)]
pub struct TypeDoc {
    pub name: String,
    pub file: PathBuf,
    pub extends: Vec<String>,
    pub kinds: Vec<Kind>,
    pub links: Vec<String>,
    pub rules: Vec<Rule>,
    pub defaults: Vec<DefaultRule>,
    pub kpis: Vec<Kpi>,
    pub extensions: Vec<Extension>,
    /// Refusals found while reading.
    pub found: Vec<Refusal>,
}

#[derive(Debug, Clone)]
pub struct Kind {
    pub name: String,
    pub block: String,
    /// The kind this kind refines.
    pub is: Option<String>,
    /// The dimensions of the block the kind fixes.
    pub fixed: Vec<(String, Node)>,
    pub attributes: Vec<Attribute>,
}

impl Kind {
    pub fn fixes(&self, field: &str) -> Option<&Node> {
        self.fixed.iter().find(|(k, _)| k == field).map(|(_, v)| v)
    }

    pub fn attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.iter().find(|a| a.name == name)
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub ty: AttributeType,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub enum AttributeType {
    String,
    Integer,
    Number,
    Boolean,
    Duration,
    Enum(Vec<String>),
}

impl AttributeType {
    /// Why `value` is no value of this type.
    pub fn fault(&self, value: &Node) -> Option<String> {
        let fault = match self {
            AttributeType::String if value.as_str().is_none() => "is not a string".to_string(),
            AttributeType::Integer if value.as_i64().is_none() => "is not an integer".to_string(),
            AttributeType::Number if !value.as_f64().is_some_and(f64::is_finite) => {
                "is not a number".to_string()
            }
            AttributeType::Boolean if value.as_bool().is_none() => {
                "is neither true nor false".to_string()
            }
            AttributeType::Duration if value.as_str().and_then(duration_seconds).is_none() => {
                blocks::not_a_duration().to_string()
            }
            AttributeType::Enum(values)
                if !value.scalar_text().is_some_and(|v| values.contains(&v)) =>
            {
                format!("is none of {}", values.join(", "))
            }
            _ => return None,
        };
        Some(fault)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Same {
    Unit,
    Parent,
}

/// A selector: every term must match. A term is a key and the values any of which matches.
#[derive(Debug, Clone, Default)]
pub struct Selector {
    pub terms: Vec<(String, Vec<Node>)>,
    pub same: Option<Same>,
}

impl Selector {
    /// Every value written for `key`, as names.
    pub fn names(&self, key: &str) -> Vec<String> {
        self.terms
            .iter()
            .filter(|(k, _)| k == key)
            .flat_map(|(_, values)| values.iter().map(Node::text))
            .collect()
    }

    /// The selector in one normal form: keys in order, the values of a key in order. Two
    /// selectors that pick the same parts by the same terms have the same text.
    pub fn text(&self) -> String {
        let mut terms: Vec<String> = self
            .terms
            .iter()
            .map(|(key, values)| {
                let mut values: Vec<String> = values.iter().map(Node::text).collect();
                values.sort();
                values.dedup();
                format!("{key}: {}", values.join("|"))
            })
            .collect();
        match self.same {
            Some(Same::Unit) => terms.push("same: unit".to_string()),
            Some(Same::Parent) => terms.push("same: parent".to_string()),
            None => {}
        }
        terms.sort();
        terms.dedup();
        format!("{{{}}}", terms.join(", "))
    }
}

/// The normal form of a selector that may be absent; an absent one picks every part.
pub fn selector_text(selector: Option<&Selector>) -> String {
    selector.map_or_else(|| Selector::default().text(), Selector::text)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    In,
    Out,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Unit,
    Parent,
    Organization,
}

/// A bound of a `value` rule: a number, or a duration in seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bound {
    Number(f64),
    Seconds(u64),
}

impl Bound {
    pub fn of(node: &Node) -> Option<Bound> {
        match node {
            Node::Int(_) | Node::Float(_) => node.as_f64().map(Bound::Number),
            Node::Str(s) => duration_seconds(s).map(Bound::Seconds),
            _ => None,
        }
    }

    /// None when one is a number and the other a duration.
    pub fn compare(self, other: Bound) -> Option<Ordering> {
        match (self, other) {
            (Bound::Number(a), Bound::Number(b)) => a.partial_cmp(&b),
            (Bound::Seconds(a), Bound::Seconds(b)) => Some(a.cmp(&b)),
            _ => None,
        }
    }
}

/// What a `value` rule allows.
#[derive(Debug, Clone, Default)]
pub struct Limits {
    pub min: Option<Node>,
    pub max: Option<Node>,
    pub one_of: Option<Vec<Node>>,
}

impl Limits {
    /// Why `value` is outside these limits.
    pub fn fault(&self, value: &Node) -> Option<String> {
        if let Some(allowed) = &self.one_of
            && !allowed.iter().any(|a| a.same(value))
        {
            let allowed: Vec<String> = allowed.iter().map(Node::text).collect();
            return Some(format!("is none of {}", allowed.join(", ")));
        }
        for (bound, wrong, word) in [
            (&self.min, Ordering::Less, "below min"),
            (&self.max, Ordering::Greater, "above max"),
        ] {
            let Some(bound) = bound else { continue };
            let order = Bound::of(value)
                .zip(Bound::of(bound))
                .and_then(|(v, b)| v.compare(b));
            match order {
                None => return Some(format!("cannot be compared with {}", bound.text())),
                Some(order) if order == wrong => {
                    return Some(format!("is {word} {}", bound.text()));
                }
                Some(_) => {}
            }
        }
        None
    }

    /// Whether some value satisfies all of `limits` at once.
    pub fn satisfiable(limits: &[&Limits]) -> bool {
        let lists: Vec<&Vec<Node>> = limits.iter().filter_map(|l| l.one_of.as_ref()).collect();
        if let Some(first) = lists.first() {
            return first
                .iter()
                .any(|v| limits.iter().all(|l| l.fault(v).is_none()));
        }
        let bounds = |pick: fn(&Limits) -> &Option<Node>| -> Vec<Option<Bound>> {
            limits
                .iter()
                .filter_map(|l| pick(l).as_ref())
                .map(Bound::of)
                .collect()
        };
        let (mins, maxs) = (bounds(|l| &l.min), bounds(|l| &l.max));
        let all: Vec<Option<Bound>> = mins.iter().chain(&maxs).copied().collect();
        let comparable = all.iter().all(|a| {
            a.zip(all[0])
                .is_some_and(|(a, first)| a.compare(first).is_some())
        });
        comparable
            && mins.iter().flatten().all(|min| {
                maxs.iter()
                    .flatten()
                    .all(|max| min.compare(*max) != Some(Ordering::Greater))
            })
    }

    pub fn text(&self) -> String {
        let mut parts = Vec::new();
        if let Some(min) = &self.min {
            parts.push(format!("min {}", min.text()));
        }
        if let Some(max) = &self.max {
            parts.push(format!("max {}", max.text()));
        }
        if let Some(allowed) = &self.one_of {
            let allowed: Vec<String> = allowed.iter().map(Node::text).collect();
            parts.push(format!("one of {}", allowed.join(", ")));
        }
        parts.join(", ")
    }
}

#[derive(Debug, Clone)]
pub enum Form {
    /// An entry with only `id` and `gate`: it restates the gate of a rule of an extended type.
    Restate,
    /// A form outside the vocabulary; the refusal is in [`TypeDoc::found`].
    Unknown,
    Parts {
        matching: Selector,
        per: Option<Selector>,
        min: Option<u64>,
        max: Option<u64>,
    },
    Links {
        each: Selector,
        carries: Vec<String>,
        direction: Direction,
        other: Option<Selector>,
        min: Option<u64>,
        max: Option<u64>,
    },
    Forbid {
        carries: Vec<String>,
        from: Selector,
        to: Selector,
    },
    Separate {
        carries: Vec<String>,
        to: Option<Selector>,
    },
    Within {
        carries: Vec<String>,
        scope: Scope,
    },
    Span {
        each: Option<Selector>,
        max: u64,
        seats: bool,
    },
    Depth {
        max: u64,
    },
    Value {
        each: Selector,
        field: String,
        limits: Limits,
        required: bool,
    },
}

impl Form {
    pub fn name(&self) -> &'static str {
        match self {
            Form::Restate => "restatement",
            Form::Unknown => "unknown",
            Form::Parts { .. } => "parts",
            Form::Links { .. } => "links",
            Form::Forbid { .. } => "forbid",
            Form::Separate { .. } => "separate",
            Form::Within { .. } => "within",
            Form::Span { .. } => "span",
            Form::Depth { .. } => "depth",
            Form::Value { .. } => "value",
        }
    }

    /// The keys the form has besides `id`, `summary`, `gate` and `rule`.
    fn keys(name: &str) -> Option<&'static [&'static str]> {
        let keys: &'static [&'static str] = match name {
            "parts" => &["match", "per", "min", "max"],
            "links" => &["each", "carries", "direction", "other", "min", "max"],
            "forbid" => &["carries", "from", "to"],
            "separate" => &["carries", "to"],
            "within" => &["carries", "scope"],
            "span" => &["each", "max", "count"],
            "depth" => &["max"],
            "value" => &["each", "field", "min", "max", "one_of", "required"],
            _ => return None,
        };
        Some(keys)
    }

    pub fn selectors(&self) -> Vec<&Selector> {
        match self {
            Form::Parts { matching, per, .. } => std::iter::once(matching).chain(per).collect(),
            Form::Links { each, other, .. } => std::iter::once(each).chain(other).collect(),
            Form::Forbid { from, to, .. } => vec![from, to],
            Form::Separate { to, .. } => to.iter().collect(),
            Form::Span { each, .. } => each.iter().collect(),
            Form::Value { each, .. } => vec![each],
            Form::Restate | Form::Unknown | Form::Within { .. } | Form::Depth { .. } => vec![],
        }
    }

    pub fn carries(&self) -> &[String] {
        match self {
            Form::Links { carries, .. }
            | Form::Forbid { carries, .. }
            | Form::Separate { carries, .. }
            | Form::Within { carries, .. } => carries,
            _ => &[],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub id: String,
    pub summary: Option<String>,
    /// The gate as written; a rule that writes none is a gate.
    pub gate: Option<bool>,
    pub form: Form,
}

impl Rule {
    /// Whether the entry declares a rule, as opposed to restating one.
    pub fn declares(&self) -> bool {
        !matches!(self.form, Form::Restate | Form::Unknown)
    }
}

#[derive(Debug, Clone)]
pub struct DefaultRule {
    pub id: String,
    pub summary: Option<String>,
    pub each: Selector,
    pub carries: String,
    pub to: Selector,
}

#[derive(Debug, Clone)]
pub struct Kpi {
    pub id: String,
    /// None for an entry that only sets the target of a KPI of an extended type.
    pub block: Option<String>,
    pub of_kind: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Extension {
    pub domain: String,
    pub extends: String,
    pub path: String,
}

fn map<'a>(node: &'a Node, at: &str) -> Result<&'a [(String, Node)], String> {
    match node {
        Node::Map(entries) => Ok(entries),
        _ => Err(format!("{at}: expected a map")),
    }
}

fn list<'a>(node: &'a Node, at: &str) -> Result<&'a [Node], String> {
    match node {
        Node::Seq(items) => Ok(items),
        _ => Err(format!("{at}: expected a list")),
    }
}

fn need<'a>(node: &'a Node, key: &str, at: &str) -> Result<&'a Node, String> {
    node.get(key)
        .ok_or_else(|| format!("{at}: missing key `{key}`"))
}

fn string(node: &Node, at: &str) -> Result<String, String> {
    node.as_str()
        .map(str::to_string)
        .ok_or_else(|| format!("{at}: expected a string"))
}

fn optional_string(node: &Node, key: &str, at: &str) -> Result<Option<String>, String> {
    node.get(key)
        .map(|v| string(v, &format!("{at}.{key}")))
        .transpose()
}

fn boolean(node: &Node, at: &str) -> Result<bool, String> {
    node.as_bool()
        .ok_or_else(|| format!("{at}: expected true or false"))
}

fn count(node: &Node, key: &str, at: &str) -> Result<Option<u64>, String> {
    node.get(key)
        .map(|v| {
            v.as_i64()
                .and_then(|n| u64::try_from(n).ok())
                .ok_or_else(|| format!("{at}.{key}: expected a whole number of at least 0"))
        })
        .transpose()
}

/// One scalar or a list of scalars.
fn values(node: &Node, at: &str) -> Result<Vec<Node>, String> {
    match node {
        Node::Seq(items) if items.iter().all(Node::is_scalar) => Ok(items.clone()),
        scalar if scalar.is_scalar() => Ok(vec![scalar.clone()]),
        _ => Err(format!("{at}: expected a value or a list of values")),
    }
}

fn names(node: &Node, at: &str) -> Result<Vec<String>, String> {
    Ok(values(node, at)?.iter().map(Node::text).collect())
}

fn one_word(node: &Node, key: &str, words: &[&str], at: &str) -> Result<String, String> {
    let word = string(need(node, key, at)?, &format!("{at}.{key}"))?;
    if words.contains(&word.as_str()) {
        Ok(word)
    } else {
        Err(format!(
            "{at}.{key}: `{word}` is none of {}",
            words.join(", ")
        ))
    }
}

/// A snake case identifier.
fn identifier(node: &Node, at: &str) -> Result<String, String> {
    let id = string(need(node, "id", at)?, &format!("{at}.id"))?;
    let snake = id.starts_with(|c: char| c.is_ascii_lowercase())
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if snake {
        Ok(id)
    } else {
        Err(format!("{at}.id: `{id}` is not snake case"))
    }
}

struct Reader<'a> {
    owner: &'a str,
    found: Vec<Refusal>,
}

impl Reader<'_> {
    fn subject(&self, item: &str) -> String {
        format!("{}.{item}", self.owner)
    }

    fn unknown_keys(&mut self, node: &Node, known: &[&[&str]], item: &str, what: &str) {
        for (key, _) in node.entries() {
            if !known.iter().any(|keys| keys.contains(&key.as_str())) {
                self.found.push(refusal(
                    "unknown_key",
                    self.subject(item),
                    format!("{what} has no key `{key}`"),
                ));
            }
        }
    }

    fn selector(
        &mut self,
        node: &Node,
        at: &str,
        item: &str,
        same: bool,
    ) -> Result<Selector, String> {
        let mut selector = Selector::default();
        if matches!(node, Node::Null) {
            return Ok(selector);
        }
        for (key, value) in map(node, at)? {
            let at = format!("{at}.{key}");
            match key.as_str() {
                "where" => {
                    for (field, value) in map(value, &at)? {
                        let term = values(value, &format!("{at}.{field}"))?;
                        selector.terms.push((field.clone(), term));
                    }
                }
                "same" if same => {
                    selector.same = Some(match string(value, &at)?.as_str() {
                        "unit" => Same::Unit,
                        "parent" => Same::Parent,
                        other => return Err(format!("{at}: `{other}` is none of unit, parent")),
                    });
                }
                "same" => self.found.push(refusal(
                    "unknown_key",
                    self.subject(item),
                    "`same` is a key of the `to` of a default only",
                )),
                _ => selector.terms.push((key.clone(), values(value, &at)?)),
            }
        }
        Ok(selector)
    }

    fn optional_selector(
        &mut self,
        node: &Node,
        key: &str,
        at: &str,
        item: &str,
    ) -> Result<Option<Selector>, String> {
        node.get(key)
            .map(|v| self.selector(v, &format!("{at}.{key}"), item, false))
            .transpose()
    }

    fn kind(&mut self, name: &str, node: &Node) -> Result<Kind, String> {
        let at = format!("kinds.{name}");
        map(node, &at)?;
        let mut kind = Kind {
            name: name.to_string(),
            block: string(need(node, "block", &at)?, &format!("{at}.block"))?,
            is: optional_string(node, "is", &at)?,
            fixed: Vec::new(),
            attributes: Vec::new(),
        };
        optional_string(node, "summary", &at)?;
        for (key, value) in node.entries() {
            match key.as_str() {
                "block" | "is" | "summary" => {}
                "attributes" => {
                    for (attribute, spec) in map(value, &format!("{at}.attributes"))? {
                        let item = format!("{name}.{attribute}");
                        kind.attributes
                            .push(self.attribute(attribute, spec, &item)?);
                    }
                }
                _ => kind.fixed.push((key.clone(), value.clone())),
            }
        }
        Ok(kind)
    }

    fn attribute(&mut self, name: &str, node: &Node, item: &str) -> Result<Attribute, String> {
        let at = format!("kinds.{item}");
        map(node, &at)?;
        self.unknown_keys(node, &[ATTRIBUTE_KEYS], item, "an attribute");
        optional_string(node, "summary", &at)?;
        let words = ["string", "integer", "number", "boolean", "duration", "enum"];
        let ty = match one_word(node, "type", &words, &at)?.as_str() {
            "string" => AttributeType::String,
            "integer" => AttributeType::Integer,
            "number" => AttributeType::Number,
            "boolean" => AttributeType::Boolean,
            "duration" => AttributeType::Duration,
            _ => AttributeType::Enum(names(need(node, "values", &at)?, &format!("{at}.values"))?),
        };
        let required = node
            .get("required")
            .map(|v| boolean(v, &format!("{at}.required")))
            .transpose()?
            .unwrap_or(false);
        Ok(Attribute {
            name: name.to_string(),
            ty,
            required,
        })
    }

    fn rule(&mut self, node: &Node, at: &str) -> Result<Rule, String> {
        map(node, at)?;
        let id = identifier(node, at)?;
        let mut rule = Rule {
            summary: optional_string(node, "summary", at)?,
            gate: node
                .get("gate")
                .map(|v| boolean(v, &format!("{at}.gate")))
                .transpose()?,
            form: Form::Restate,
            id,
        };
        let Some(form) = node.get("rule") else {
            let only = ["id", "gate", "summary"];
            if node
                .entries()
                .iter()
                .any(|(k, _)| !only.contains(&k.as_str()))
            {
                return Err(format!("{at}: missing key `rule`"));
            }
            need(node, "gate", at)?;
            return Ok(rule);
        };
        let form = string(form, &format!("{at}.rule"))?;
        let Some(keys) = Form::keys(&form) else {
            self.found.push(refusal(
                "unknown_rule_form",
                self.subject(&rule.id),
                format!("`{form}` is no rule form"),
            ));
            rule.form = Form::Unknown;
            return Ok(rule);
        };
        let id = rule.id.clone();
        self.unknown_keys(
            node,
            &[RULE_KEYS, keys],
            &id,
            &format!("rule form `{form}`"),
        );
        let carries = |min: usize| -> Result<Vec<String>, String> {
            let here = format!("{at}.carries");
            let mut carried = names(need(node, "carries", at)?, &here)?;
            carried.sort();
            carried.dedup();
            if carried.len() < min {
                return Err(format!("{here}: expected {min} different names or more"));
            }
            Ok(carried)
        };
        rule.form = match form.as_str() {
            "parts" => Form::Parts {
                matching: self.selector(
                    need(node, "match", at)?,
                    &format!("{at}.match"),
                    &id,
                    false,
                )?,
                per: self.optional_selector(node, "per", at, &id)?,
                min: count(node, "min", at)?,
                max: count(node, "max", at)?,
            },
            "links" => Form::Links {
                each: self
                    .optional_selector(node, "each", at, &id)?
                    .unwrap_or_default(),
                carries: carries(1)?,
                direction: match one_word(node, "direction", &["in", "out"], at)?.as_str() {
                    "in" => Direction::In,
                    _ => Direction::Out,
                },
                other: self.optional_selector(node, "other", at, &id)?,
                min: count(node, "min", at)?,
                max: count(node, "max", at)?,
            },
            "forbid" => Form::Forbid {
                carries: carries(1)?,
                from: self
                    .optional_selector(node, "from", at, &id)?
                    .unwrap_or_default(),
                to: self
                    .optional_selector(node, "to", at, &id)?
                    .unwrap_or_default(),
            },
            "separate" => Form::Separate {
                carries: carries(2)?,
                to: self.optional_selector(node, "to", at, &id)?,
            },
            "within" => Form::Within {
                carries: carries(1)?,
                scope: match one_word(node, "scope", &["unit", "parent", "organization"], at)?
                    .as_str()
                {
                    "unit" => Scope::Unit,
                    "parent" => Scope::Parent,
                    _ => Scope::Organization,
                },
            },
            "span" => Form::Span {
                each: self.optional_selector(node, "each", at, &id)?,
                max: count(node, "max", at)?.ok_or_else(|| format!("{at}: missing key `max`"))?,
                seats: match node.get("count") {
                    None => false,
                    Some(_) => one_word(node, "count", &["positions", "seats"], at)? == "seats",
                },
            },
            "depth" => Form::Depth {
                max: count(node, "max", at)?.ok_or_else(|| format!("{at}: missing key `max`"))?,
            },
            _ => {
                let bound = |key: &str| -> Result<Option<Node>, String> {
                    match node.get(key) {
                        Some(v) if Bound::of(v).is_none() => {
                            Err(format!("{at}.{key}: expected a number or a duration"))
                        }
                        other => Ok(other.cloned()),
                    }
                };
                Form::Value {
                    each: self
                        .optional_selector(node, "each", at, &id)?
                        .unwrap_or_default(),
                    field: string(need(node, "field", at)?, &format!("{at}.field"))?,
                    limits: Limits {
                        min: bound("min")?,
                        max: bound("max")?,
                        one_of: node
                            .get("one_of")
                            .map(|v| values(v, &format!("{at}.one_of")))
                            .transpose()?,
                    },
                    required: node
                        .get("required")
                        .map(|v| boolean(v, &format!("{at}.required")))
                        .transpose()?
                        .unwrap_or(false),
                }
            }
        };
        Ok(rule)
    }

    fn default_rule(&mut self, node: &Node, at: &str) -> Result<DefaultRule, String> {
        map(node, at)?;
        let id = identifier(node, at)?;
        self.unknown_keys(node, &[DEFAULT_KEYS], &id, "a default");
        Ok(DefaultRule {
            summary: optional_string(node, "summary", at)?,
            each: self
                .optional_selector(node, "each", at, &id)?
                .unwrap_or_default(),
            carries: string(need(node, "carries", at)?, &format!("{at}.carries"))?,
            to: self.selector(need(node, "to", at)?, &format!("{at}.to"), &id, true)?,
            id,
        })
    }

    fn kpi(&mut self, node: &Node, at: &str) -> Result<Kpi, String> {
        map(node, at)?;
        let id = identifier(node, at)?;
        self.unknown_keys(node, &[KPI_KEYS], &id, "a KPI");
        if let Some(target) = node.get("target")
            && !target.is_scalar()
        {
            return Err(format!("{at}.target: expected a single value"));
        }
        let only_target = node
            .entries()
            .iter()
            .all(|(k, _)| k == "id" || k == "target");
        if only_target && node.get("target").is_some() {
            return Ok(Kpi {
                id,
                block: None,
                of_kind: None,
            });
        }
        string(need(node, "summary", at)?, &format!("{at}.summary"))?;
        string(need(node, "unit", at)?, &format!("{at}.unit"))?;
        one_word(
            node,
            "measures",
            &["correctness", "completeness", "quality"],
            at,
        )?;
        one_word(node, "direction", &["higher", "lower", "target"], at)?;
        Ok(Kpi {
            block: Some(string(need(node, "block", at)?, &format!("{at}.block"))?),
            of_kind: optional_string(node, "of_kind", at)?,
            id,
        })
    }

    fn extension(&mut self, node: &Node, at: &str) -> Result<Extension, String> {
        map(node, at)?;
        let text = |key: &str| string(need(node, key, at)?, &format!("{at}.{key}"));
        let extension = Extension {
            domain: text("domain")?,
            extends: text("extends")?,
            path: text("path")?,
        };
        self.unknown_keys(node, &[EXTENSION_KEYS], &extension.domain, "an extension");
        Ok(extension)
    }
}

/// Reads the type document `doc`, whose `type` is `name`.
pub fn read(doc: &Node, name: &str, file: &Path) -> Result<TypeDoc, String> {
    map(doc, "the document")?;
    let named_twice =
        |path: &Vec<String>| path.len() == 2 && (path[0] == "kinds" || path[0] == "links");
    if let Some(path) = doc.repeated_keys().iter().find(|p| !named_twice(p)) {
        return Err(format!("{}: the key is written twice", path.join(".")));
    }
    let mut reader = Reader {
        owner: name,
        found: Vec::new(),
    };
    let mut doc_type = TypeDoc {
        name: name.to_string(),
        file: file.to_path_buf(),
        extends: vec!["base".to_string()],
        ..TypeDoc::default()
    };
    for (key, value) in doc.entries() {
        if !KEYS.contains(&key.as_str()) {
            reader.found.push(refusal(
                "unknown_key",
                name,
                format!("a type document has no key `{key}`"),
            ));
            continue;
        }
        match key.as_str() {
            "format" | "type" => {}
            "origin" | "summary" => {
                string(value, key)?;
            }
            "extends" => {
                let extended = names(value, "extends")?;
                if !extended.is_empty() {
                    doc_type.extends = extended;
                }
            }
            "vocabulary" => {
                for (word, shown) in map(value, "vocabulary")? {
                    string(shown, &format!("vocabulary.{word}"))?;
                }
            }
            "kinds" => {
                for (kind, spec) in map(value, "kinds")? {
                    doc_type.kinds.push(reader.kind(kind, spec)?);
                }
            }
            "links" => {
                for (link, spec) in map(value, "links")? {
                    let at = format!("links.{link}");
                    if !matches!(spec, Node::Null) {
                        map(spec, &at)?;
                        optional_string(spec, "summary", &at)?;
                        reader.unknown_keys(spec, &[&["summary"]], link, "a link");
                    }
                    doc_type.links.push(link.clone());
                }
            }
            "rules" => {
                for (i, entry) in list(value, "rules")?.iter().enumerate() {
                    doc_type
                        .rules
                        .push(reader.rule(entry, &format!("rules[{i}]"))?);
                }
            }
            "defaults" => {
                for (i, entry) in list(value, "defaults")?.iter().enumerate() {
                    let at = format!("defaults[{i}]");
                    doc_type.defaults.push(reader.default_rule(entry, &at)?);
                }
            }
            "kpis" => {
                for (i, entry) in list(value, "kpis")?.iter().enumerate() {
                    doc_type
                        .kpis
                        .push(reader.kpi(entry, &format!("kpis[{i}]"))?);
                }
            }
            _ => {
                for (i, entry) in list(value, "extensions")?.iter().enumerate() {
                    let at = format!("extensions[{i}]");
                    doc_type.extensions.push(reader.extension(entry, &at)?);
                }
            }
        }
    }
    doc_type.found = reader.found;
    Ok(doc_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::parse;

    fn selector(text: &str) -> Selector {
        let mut reader = Reader {
            owner: "t",
            found: Vec::new(),
        };
        reader
            .selector(&parse(text).unwrap(), "s", "r", true)
            .unwrap()
    }

    #[test]
    fn selectors_are_compared_in_one_normal_form() {
        let a = selector("{ block: position, held_by: [mixed, agent] }");
        let b = selector("{ where: { held_by: [agent, mixed] }, block: [position] }");
        assert_eq!(a.text(), b.text());
        assert_eq!(a.text(), "{block: position, held_by: agent|mixed}");
        assert_ne!(a.text(), selector("{ block: position }").text());
        assert_eq!(selector_text(None), "{}");
    }

    #[test]
    fn limits_hold_together_or_not() {
        let limits = |text: &str| {
            let node = parse(text).unwrap();
            Limits {
                min: node.get("min").cloned(),
                max: node.get("max").cloned(),
                one_of: node.get("one_of").map(|v| values(v, "x").unwrap()),
            }
        };
        let sat = |a: &str, b: &str| Limits::satisfiable(&[&limits(a), &limits(b)]);
        assert!(sat("{ min: 1, max: 4 }", "{ min: 4 }"));
        assert!(!sat("{ max: 3 }", "{ min: 4 }"));
        assert!(sat("{ min: 1d }", "{ max: 1w }"));
        assert!(!sat("{ min: 2w }", "{ max: 7d }"));
        assert!(!sat("{ min: 2 }", "{ max: 7d }"));
        assert!(sat("{ one_of: [a, b] }", "{ one_of: [b, c] }"));
        assert!(!sat("{ one_of: [a] }", "{ one_of: [b] }"));
        assert!(!sat("{ one_of: [1, 2] }", "{ min: 3 }"));
        assert!(!Limits::satisfiable(&[&limits("{ min: 5, max: 2 }")]));
        assert!(Limits::satisfiable(&[&limits("{}")]));
    }
}
