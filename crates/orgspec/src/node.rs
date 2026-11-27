//! A YAML document as a tree that keeps the order of keys and keeps a key written twice.

use serde::de::{Deserialize, Deserializer, Error, MapAccess, SeqAccess, Visitor};
use std::fmt;
use std::path::Path;

/// One value of a YAML document.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Seq(Vec<Node>),
    Map(Vec<(String, Node)>),
}

struct NodeVisitor;

impl<'de> Visitor<'de> for NodeVisitor {
    type Value = Node;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a YAML value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Node, E> {
        Ok(Node::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Node, E> {
        Ok(Node::Int(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Node, E> {
        Ok(i64::try_from(v).map_or(Node::Float(v as f64), Node::Int))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Node, E> {
        Ok(Node::Float(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Node, E> {
        Ok(Node::Str(v.to_string()))
    }

    fn visit_unit<E>(self) -> Result<Node, E> {
        Ok(Node::Null)
    }

    fn visit_none<E>(self) -> Result<Node, E> {
        Ok(Node::Null)
    }

    fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Node, D::Error> {
        Node::deserialize(d)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Node, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element()? {
            items.push(item);
        }
        Ok(Node::Seq(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Node, A::Error> {
        let mut entries = Vec::new();
        while let Some(key) = map.next_key::<Node>()? {
            let key = key
                .scalar_text()
                .ok_or_else(|| A::Error::custom("a key is not a scalar"))?;
            entries.push((key, map.next_value()?));
        }
        Ok(Node::Map(entries))
    }
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Node, D::Error> {
        d.deserialize_any(NodeVisitor)
    }
}

/// Parses one YAML document.
pub fn parse(text: &str) -> Result<Node, String> {
    serde_yaml_ng::from_str(text).map_err(|e| format!("not YAML: {e}"))
}

/// Reads and parses one YAML file.
pub fn load(path: &Path) -> Result<Node, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot be read: {e}"))?;
    parse(&text)
}

/// The seconds of a duration written `<integer><s|m|h|d|w>`.
pub fn duration_seconds(text: &str) -> Option<u64> {
    let unit = text.chars().last()?;
    let digits = &text[..text.len() - unit.len_utf8()];
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let factor = match unit {
        's' => 1,
        'm' => 60,
        'h' => 3_600,
        'd' => 86_400,
        'w' => 604_800,
        _ => return None,
    };
    digits.parse::<u64>().ok()?.checked_mul(factor)
}

impl Node {
    /// The entries of a map; nothing for any other value.
    pub fn entries(&self) -> &[(String, Node)] {
        match self {
            Node::Map(entries) => entries,
            _ => &[],
        }
    }

    /// The first value written under `key`.
    pub fn get(&self, key: &str) -> Option<&Node> {
        self.entries()
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Node::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Node::Int(v) => Some(*v as f64),
            Node::Float(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Node::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Node::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn is_scalar(&self) -> bool {
        !matches!(self, Node::Null | Node::Seq(_) | Node::Map(_))
    }

    /// A scalar as text, the way a name is written.
    pub fn scalar_text(&self) -> Option<String> {
        self.is_scalar().then(|| self.text())
    }

    /// The value as it is shown in a message.
    pub fn text(&self) -> String {
        match self {
            Node::Null => "null".to_string(),
            Node::Bool(v) => v.to_string(),
            Node::Int(v) => v.to_string(),
            Node::Float(v) if v.fract() == 0.0 && v.abs() < 1e15 => format!("{}", *v as i64),
            Node::Float(v) => v.to_string(),
            Node::Str(s) => s.clone(),
            Node::Seq(items) => {
                let items: Vec<String> = items.iter().map(Node::text).collect();
                format!("[{}]", items.join(", "))
            }
            Node::Map(entries) => {
                let entries: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| format!("{k}: {}", v.text()))
                    .collect();
                format!("{{{}}}", entries.join(", "))
            }
        }
    }

    /// Whether two scalars state the same value: numbers by value, durations by their seconds.
    pub fn same(&self, other: &Node) -> bool {
        match (self, other) {
            (Node::Str(a), Node::Str(b)) => {
                a == b
                    || matches!(
                        (duration_seconds(a), duration_seconds(b)),
                        (Some(x), Some(y)) if x == y
                    )
            }
            (Node::Bool(a), Node::Bool(b)) => a == b,
            _ => matches!((self.as_f64(), other.as_f64()), (Some(x), Some(y)) if x == y),
        }
    }

    /// Whether this is a `{ value, vary }` wrapper: a map with a key `value` and no key besides
    /// `value` and `vary`.
    pub fn is_wrapper(&self) -> bool {
        let entries = self.entries();
        entries.iter().any(|(k, _)| k == "value")
            && entries.iter().all(|(k, _)| k == "value" || k == "vary")
    }

    /// The value with every `{ value, vary }` wrapper replaced by its `value`.
    pub fn unwrapped(&self) -> Node {
        match self {
            Node::Map(_) if self.is_wrapper() => {
                self.get("value").map_or(Node::Null, Node::unwrapped)
            }
            Node::Map(entries) => Node::Map(
                entries
                    .iter()
                    .map(|(k, v)| (k.clone(), v.unwrapped()))
                    .collect(),
            ),
            Node::Seq(items) => Node::Seq(items.iter().map(Node::unwrapped).collect()),
            other => other.clone(),
        }
    }

    /// What is wrong with the first `vary` inside this value that is neither a list of scalars
    /// nor `{ min, max, step }`.
    pub fn vary_fault(&self) -> Option<String> {
        match self {
            Node::Map(entries) if self.is_wrapper() => {
                let fine = match self.get("vary") {
                    None => true,
                    Some(Node::Seq(items)) => items.iter().all(Node::is_scalar),
                    Some(Node::Map(range)) => range.iter().all(|(k, v)| {
                        ["min", "max", "step"].contains(&k.as_str()) && v.is_scalar()
                    }),
                    Some(_) => false,
                };
                if fine {
                    entries.iter().find_map(|(_, v)| v.vary_fault())
                } else {
                    Some("`vary` is neither a list of values nor `{ min, max, step }`".to_string())
                }
            }
            Node::Map(entries) => entries.iter().find_map(|(_, v)| v.vary_fault()),
            Node::Seq(items) => items.iter().find_map(Node::vary_fault),
            _ => None,
        }
    }

    /// The path of every key that one map of this value writes twice.
    pub fn repeated_keys(&self) -> Vec<Vec<String>> {
        let mut out = Vec::new();
        self.collect_repeated(&mut Vec::new(), &mut out);
        out
    }

    fn collect_repeated(&self, path: &mut Vec<String>, out: &mut Vec<Vec<String>>) {
        match self {
            Node::Map(entries) => {
                for (i, (key, value)) in entries.iter().enumerate() {
                    path.push(key.clone());
                    if entries[..i].iter().any(|(k, _)| k == key) {
                        out.push(path.clone());
                    }
                    value.collect_repeated(path, out);
                    path.pop();
                }
            }
            Node::Seq(items) => {
                for (i, item) in items.iter().enumerate() {
                    path.push(format!("[{i}]"));
                    item.collect_repeated(path, out);
                    path.pop();
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_duration_is_an_integer_and_a_unit() {
        assert_eq!(duration_seconds("90s"), Some(90));
        assert_eq!(duration_seconds("15m"), Some(900));
        assert_eq!(duration_seconds("24h"), Some(86_400));
        assert_eq!(duration_seconds("7d"), Some(604_800));
        assert_eq!(duration_seconds("2w"), Some(1_209_600));
        for bad in ["", "d", "7", "7 d", "1.5h", "-2d", "7y", "seven"] {
            assert_eq!(duration_seconds(bad), None, "{bad}");
        }
    }

    #[test]
    fn a_wrapper_is_read_as_its_value() {
        let doc = parse("seats: { value: 4, vary: { min: 2, max: 8 } }\nevery: { value: 7d }\n")
            .unwrap()
            .unwrapped();
        assert_eq!(doc.get("seats"), Some(&Node::Int(4)));
        assert_eq!(doc.get("every"), Some(&Node::Str("7d".into())));
        let other = parse("a: { value: 1, note: x }").unwrap().unwrapped();
        assert!(matches!(other.get("a"), Some(Node::Map(_))));
    }

    #[test]
    fn a_bad_vary_is_named() {
        assert!(
            parse("a: { value: 1, vary: [1, 2] }")
                .unwrap()
                .vary_fault()
                .is_none()
        );
        assert!(
            parse("a: { value: 1, vary: 3 }")
                .unwrap()
                .vary_fault()
                .is_some()
        );
        assert!(
            parse("a: { value: 1, vary: { from: 1 } }")
                .unwrap()
                .vary_fault()
                .is_some()
        );
    }

    #[test]
    fn a_key_written_twice_is_kept_and_found() {
        let doc = parse("kinds:\n  ward: { block: unit }\n  ward: { block: role }\n").unwrap();
        assert_eq!(doc.get("kinds").unwrap().entries().len(), 2);
        assert_eq!(
            doc.repeated_keys(),
            vec![vec!["kinds".to_string(), "ward".to_string()]]
        );
    }

    #[test]
    fn scalars_compare_by_value() {
        assert!(Node::Int(1).same(&Node::Float(1.0)));
        assert!(Node::Str("7d".into()).same(&Node::Str("1w".into())));
        assert!(!Node::Str("true".into()).same(&Node::Bool(true)));
    }
}
