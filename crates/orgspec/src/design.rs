//! An organization design document (`orgspec-organization/1`) and what is read off it.

use crate::graph::{self, Graph};
use crate::node::Node;
use crate::types::Kind;
use crate::typeset::TypeSet;
use std::collections::{BTreeMap, BTreeSet};

pub const FORMAT: &str = "orgspec-organization/1";

const KEYS: &[&str] = &[
    "format",
    "organization",
    "type",
    "origin",
    "summary",
    "parts",
    "links",
    "border",
];

fn unwrapped(node: &Node) -> Vec<(String, Node)> {
    node.entries()
        .iter()
        .map(|(key, value)| (key.clone(), value.unwrapped()))
        .collect()
}

fn field<'a>(fields: &'a [(String, Node)], name: &str) -> Option<&'a Node> {
    fields.iter().find(|(k, _)| k == name).map(|(_, v)| v)
}

#[derive(Debug, Clone)]
pub struct Part {
    pub name: String,
    /// Every field as written, a `{ value, vary }` read as its value.
    pub fields: Vec<(String, Node)>,
    pub vary_fault: Option<String>,
}

impl Part {
    pub fn get(&self, name: &str) -> Option<&Node> {
        field(&self.fields, name)
    }

    /// The field when it is one scalar, as a name.
    pub fn text(&self, name: &str) -> Option<String> {
        self.get(name).and_then(Node::scalar_text)
    }

    pub fn block(&self) -> Option<&str> {
        self.get("block").and_then(Node::as_str)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Link {
    pub from: Option<String>,
    pub to: Option<String>,
    pub carries: Option<String>,
    /// Every key as written, a `{ value, vary }` read as its value.
    pub fields: Vec<(String, Node)>,
    pub vary_fault: Option<String>,
    /// The id of the default that added the link; none for a link the design wrote.
    pub added_by: Option<String>,
}

impl Link {
    fn read(node: &Node) -> Link {
        let fields = unwrapped(node);
        let name = |key: &str| field(&fields, key).and_then(Node::scalar_text);
        Link {
            from: name("from"),
            to: name("to"),
            carries: name("carries"),
            vary_fault: node.vary_fault(),
            added_by: None,
            fields,
        }
    }

    /// A link that the default `by` adds.
    pub fn added(from: &str, to: &str, carries: &str, by: &str) -> Link {
        let text = |value: &str| Some(value.to_string());
        Link {
            from: text(from),
            to: text(to),
            carries: text(carries),
            fields: [("from", from), ("to", to), ("carries", carries)]
                .iter()
                .map(|(key, value)| (key.to_string(), Node::Str(value.to_string())))
                .collect(),
            vary_fault: None,
            added_by: text(by),
        }
    }

    /// `<from>-><to>:<carries>`.
    pub fn name(&self) -> String {
        let shown = |end: &Option<String>| end.clone().unwrap_or_else(|| "?".to_string());
        format!(
            "{}->{}:{}",
            shown(&self.from),
            shown(&self.to),
            shown(&self.carries)
        )
    }

    pub fn get(&self, name: &str) -> Option<&Node> {
        field(&self.fields, name)
    }

    pub fn text(&self, name: &str) -> Option<String> {
        self.get(name).and_then(Node::scalar_text)
    }

    pub fn limit(&self) -> Option<f64> {
        self.get("limit").and_then(Node::as_f64)
    }
}

#[derive(Debug, Clone)]
pub struct Design {
    pub organization: String,
    /// The types the design names; `base` when it names none.
    pub types: Vec<String>,
    pub parts: Vec<Part>,
    pub links: Vec<Link>,
    pub border: Vec<String>,
    /// The top-level keys the format does not have.
    pub unknown_keys: Vec<String>,
}

impl Design {
    /// Reads a design. Fails when the document does not have the shape of the format.
    pub fn read(doc: &Node) -> Result<Design, String> {
        if !matches!(doc, Node::Map(_)) {
            return Err("the document is not a map".to_string());
        }
        match doc.get("format").map(Node::text) {
            None => return Err("missing key `format`".to_string()),
            Some(format) if format != FORMAT => {
                return Err(format!("unknown format `{format}`, expected `{FORMAT}`"));
            }
            Some(_) => {}
        }
        if let Some(path) = doc.repeated_keys().first() {
            return Err(format!("{}: the key is written twice", path.join(".")));
        }
        let organization = match doc.get("organization") {
            None => return Err("missing key `organization`".to_string()),
            Some(name) => name
                .as_str()
                .ok_or("organization: expected a string")?
                .to_string(),
        };
        let names = |key: &str| -> Result<Vec<String>, String> {
            match doc.get(key) {
                None => Ok(Vec::new()),
                Some(Node::Seq(items)) if items.iter().all(Node::is_scalar) => {
                    Ok(items.iter().map(Node::text).collect())
                }
                Some(one) if one.is_scalar() && key == "type" => Ok(vec![one.text()]),
                Some(_) => Err(format!("{key}: expected a list of names")),
            }
        };
        let mut types = names("type")?;
        if types.is_empty() {
            types.push("base".to_string());
        }
        let mut parts = Vec::new();
        match doc.get("parts") {
            None => return Err("missing key `parts`".to_string()),
            Some(Node::Map(entries)) => {
                for (name, part) in entries {
                    if !matches!(part, Node::Map(_)) {
                        return Err(format!("parts.{name}: expected a map"));
                    }
                    parts.push(Part {
                        name: name.clone(),
                        fields: unwrapped(part),
                        vary_fault: part.vary_fault(),
                    });
                }
            }
            Some(_) => return Err("parts: expected a map of parts".to_string()),
        }
        let mut links = Vec::new();
        match doc.get("links") {
            None | Some(Node::Null) => {}
            Some(Node::Seq(items)) => {
                for (i, link) in items.iter().enumerate() {
                    if !matches!(link, Node::Map(_)) {
                        return Err(format!("links[{i}]: expected a map"));
                    }
                    links.push(Link::read(link));
                }
            }
            Some(_) => return Err("links: expected a list of links".to_string()),
        }
        Ok(Design {
            organization,
            types,
            parts,
            links,
            border: names("border")?,
            unknown_keys: doc
                .entries()
                .iter()
                .map(|(key, _)| key.clone())
                .filter(|key| !KEYS.contains(&key.as_str()))
                .collect(),
        })
    }
}

/// A design with its types and the links its defaults added.
pub struct Model<'a> {
    pub design: &'a Design,
    pub set: &'a TypeSet,
    /// The links the design wrote, then the links the defaults added.
    pub links: Vec<Link>,
    parts: BTreeMap<&'a str, &'a Part>,
}

impl<'a> Model<'a> {
    pub fn new(design: &'a Design, set: &'a TypeSet) -> Model<'a> {
        Model {
            design,
            set,
            links: design.links.clone(),
            parts: design.parts.iter().map(|p| (p.name.as_str(), p)).collect(),
        }
    }

    pub fn part(&self, name: &str) -> Option<&'a Part> {
        self.parts.get(name).copied()
    }

    /// Whether `name` is a part of `block`.
    pub fn is(&self, name: &str, block: &str) -> bool {
        self.part(name).is_some_and(|p| p.block() == Some(block))
    }

    /// The kind of the part and every kind it refines; nothing when the kind is unknown or of
    /// another block.
    pub fn kinds(&self, part: &Part) -> Vec<&'a Kind> {
        let Some(kind) = part.text("kind") else {
            return Vec::new();
        };
        let chain = self.set.kind_chain(&kind);
        match chain.first() {
            Some(first) if Some(first.block.as_str()) == part.block() => chain,
            _ => Vec::new(),
        }
    }

    /// The value a field of the part has: what the part states, else what its kind fixes, else
    /// the default of the block.
    pub fn value(&self, part: &Part, name: &str) -> Option<Node> {
        if let Some(stated) = part.get(name) {
            return Some(stated.clone());
        }
        if let Some(fixed) = self.kinds(part).iter().find_map(|k| k.fixes(name)) {
            return Some(fixed.clone());
        }
        match (part.block()?, name) {
            ("position", "held_by") => Some(Node::Str("human".to_string())),
            ("position", "seats") => Some(Node::Int(1)),
            ("decision_class", "reversible") => Some(Node::Bool(true)),
            _ => None,
        }
    }

    pub fn text(&self, part: &Part, name: &str) -> Option<String> {
        self.value(part, name).and_then(|v| v.scalar_text())
    }

    pub fn seats(&self, part: &Part) -> i64 {
        self.value(part, "seats")
            .and_then(|v| v.as_i64())
            .filter(|seats| *seats >= 1)
            .unwrap_or(1)
    }

    /// The seats of a position that agents hold.
    pub fn agent_seats(&self, part: &Part) -> f64 {
        let seats = self.seats(part) as f64;
        match self.text(part, "held_by").as_deref() {
            Some("agent") => seats,
            Some("mixed") => {
                let share = self.value(part, "agent_share").and_then(|v| v.as_f64());
                seats * share.filter(|s| (0.0..=1.0).contains(s)).unwrap_or(0.5)
            }
            _ => 0.0,
        }
    }

    /// The unit of a part: of a position its `in`, of a unit itself, of a forum its `convenes`,
    /// of a budget its `for`.
    pub fn unit_of(&self, part: &Part) -> Option<String> {
        let placed_by = match part.block()? {
            "unit" => return Some(part.name.clone()),
            "position" => "in",
            "forum" => "convenes",
            "budget" => "for",
            _ => return None,
        };
        self.text(part, placed_by)
            .filter(|unit| self.is(unit, "unit"))
    }

    /// What a unit is `in`.
    pub fn parent_of(&self, unit: &str) -> Option<String> {
        self.part(unit).and_then(|p| self.text(p, "in"))
    }

    /// The organization part a part reaches through `in`.
    pub fn organization_of(&self, part: &Part) -> Option<String> {
        if part.block() == Some("organization") {
            return Some(part.name.clone());
        }
        let mut at = self.unit_of(part)?;
        let mut seen = BTreeSet::new();
        while seen.insert(at.clone()) {
            if self.is(&at, "organization") {
                return Some(at);
            }
            at = self.parent_of(&at)?;
        }
        None
    }

    /// The links that carry `carries` and whose two ends are parts: from, to, the link.
    pub fn carrying<'m>(
        &'m self,
        carries: &'m str,
    ) -> impl Iterator<Item = (&'m str, &'m str, &'m Link)> {
        self.links
            .iter()
            .filter(move |link| link.carries.as_deref() == Some(carries))
            .filter_map(|link| {
                let (from, to) = (link.from.as_deref()?, link.to.as_deref()?);
                (self.part(from).is_some() && self.part(to).is_some()).then_some((from, to, link))
            })
    }

    /// The positions that report to the position `name`, each once.
    pub fn reporting_to(&self, name: &str) -> Vec<&'a Part> {
        let from: BTreeSet<&str> = self
            .carrying("reports_to")
            .filter(|(_, to, _)| *to == name)
            .map(|(from, _, _)| from)
            .collect();
        from.iter()
            .filter_map(|from| self.part(from))
            .filter(|part| part.block() == Some("position"))
            .collect()
    }

    pub fn graph(&self, carries: &str) -> Graph {
        graph::graph(self.carrying(carries).map(|(from, to, _)| (from, to)))
    }

    /// Who holds which decision class, with the limit it holds it to; no limit is infinity.
    /// A part holds a class it decides, and a class delegated to it by a part that holds it.
    pub fn holdings(&self) -> BTreeMap<(String, String), f64> {
        type Held = BTreeMap<(String, String), f64>;
        fn raise(held: &mut Held, part: &str, class: &str, limit: f64) -> bool {
            let now = held
                .entry((part.to_string(), class.to_string()))
                .or_insert(f64::NEG_INFINITY);
            let raised = limit > *now;
            if raised {
                *now = limit;
            }
            raised
        }
        let mut held = Held::new();
        for (from, class, link) in self.carrying("decides") {
            if self.is(class, "decision_class") {
                raise(
                    &mut held,
                    from,
                    class,
                    link.limit().unwrap_or(f64::INFINITY),
                );
            }
        }
        let delegations: Vec<(&str, &str, String, Option<f64>)> = self
            .carrying("delegates")
            .filter_map(|(from, to, link)| Some((from, to, link.text("class")?, link.limit())))
            .filter(|(_, _, class, _)| self.is(class, "decision_class"))
            .collect();
        loop {
            let mut changed = false;
            for (from, to, class, limit) in &delegations {
                if let Some(&has) = held.get(&(from.to_string(), class.clone())) {
                    let passed = limit.map_or(has, |limit| limit.min(has));
                    changed |= raise(&mut held, to, class, passed);
                }
            }
            if !changed {
                return held;
            }
        }
    }
}
