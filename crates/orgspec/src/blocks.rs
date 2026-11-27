//! The blocks a design part may name and the fields each block has, as data.
//!
//! This table follows `design/building-blocks.md`. It is adjusted when the ESS blocks in
//! `blocks/` settle.

use crate::node::{Node, duration_seconds};

/// What a field of a block holds.
pub enum Ty {
    /// A string.
    Text,
    /// Any scalar.
    Scalar,
    /// An integer of at least `min`.
    Integer {
        min: i64,
    },
    Number,
    /// A number from 0 to 1.
    Share,
    Boolean,
    Duration,
    /// One of the listed words.
    Enum(&'static [&'static str]),
    /// The name of another part of one of the listed blocks; an empty list means any block.
    Ref(&'static [&'static str]),
    /// A list of strings.
    TextList,
    /// The steps of a process.
    Steps,
}

pub struct Field {
    pub name: &'static str,
    pub ty: Ty,
    pub required: bool,
}

pub struct Block {
    pub name: &'static str,
    pub fields: &'static [Field],
}

const fn field(name: &'static str, ty: Ty) -> Field {
    Field {
        name,
        ty,
        required: false,
    }
}

const fn required(name: &'static str, ty: Ty) -> Field {
    Field {
        name,
        ty,
        required: true,
    }
}

/// The fields every part accepts, whatever its block.
pub const COMMON: &[&str] = &["block", "kind", "summary", "title"];

pub const LEVELS: &[&str] = &["low", "medium", "high", "critical"];
pub const HELD_BY: &[&str] = &["human", "agent", "mixed"];
pub const ACTIVITIES: &[&str] = &["do", "decide", "review", "wait"];

/// The keys of one step of a process.
pub const STEP_KEYS: &[&str] = &["name", "role", "activity"];

/// The blocks a part of a design may name.
pub const BLOCKS: &[Block] = &[
    Block {
        name: "organization",
        fields: &[
            field(
                "form",
                Ty::Enum(&[
                    "company",
                    "public_body",
                    "association",
                    "cooperative",
                    "partnership",
                    "project",
                    "informal",
                ]),
            ),
            field("purpose", Ty::Text),
            field("in", Ty::Ref(&["organization"])),
        ],
    },
    Block {
        name: "unit",
        fields: &[
            required("in", Ty::Ref(&["organization", "unit"])),
            field(
                "nature",
                Ty::Enum(&["line", "governing", "community", "temporary"]),
            ),
            field(
                "basis",
                Ty::Enum(&[
                    "function", "offering", "market", "region", "process", "project",
                ]),
            ),
            field("purpose", Ty::Text),
        ],
    },
    Block {
        name: "role",
        fields: &[field("purpose", Ty::Text), field("expects", Ty::TextList)],
    },
    Block {
        name: "position",
        fields: &[
            required("in", Ty::Ref(&["unit"])),
            field("role", Ty::Ref(&["role"])),
            field("seats", Ty::Integer { min: 1 }),
            field("held_by", Ty::Enum(HELD_BY)),
            field("agent_share", Ty::Share),
            field("answer_delay", Ty::Duration),
        ],
    },
    Block {
        name: "decision_class",
        fields: &[
            field("reversible", Ty::Boolean),
            field("impact", Ty::Enum(LEVELS)),
        ],
    },
    Block {
        name: "policy",
        fields: &[
            field("category", Ty::Enum(&["rule", "principle", "standard"])),
            field("scope", Ty::Ref(&["organization", "unit"])),
        ],
    },
    Block {
        name: "offering",
        fields: &[field("audience", Ty::Enum(&["internal", "external"]))],
    },
    Block {
        name: "relationship",
        fields: &[field(
            "tie",
            Ty::Enum(&[
                "customer",
                "supplier",
                "partner",
                "regulator",
                "funder",
                "beneficiary",
            ]),
        )],
    },
    Block {
        name: "goal",
        fields: &[
            field("by", Ty::Text),
            field("parent", Ty::Ref(&["goal"])),
            field("measure", Ty::Text),
        ],
    },
    Block {
        name: "asset",
        fields: &[
            field(
                "class",
                Ty::Enum(&[
                    "physical",
                    "digital",
                    "information",
                    "financial",
                    "intangible",
                ]),
            ),
            field("criticality", Ty::Enum(LEVELS)),
        ],
    },
    Block {
        name: "budget",
        fields: &[
            field("amount", Ty::Number),
            field("currency", Ty::Text),
            field("period", Ty::Duration),
            field("for", Ty::Ref(&["unit"])),
        ],
    },
    Block {
        name: "process",
        fields: &[field("steps", Ty::Steps)],
    },
    Block {
        name: "agreement",
        fields: &[
            field("provider", Ty::Ref(&["unit"])),
            field("consumer", Ty::Ref(&["unit", "relationship"])),
            field("measure", Ty::Scalar),
            field("target", Ty::Scalar),
        ],
    },
    Block {
        name: "forum",
        fields: &[
            field("convenes", Ty::Ref(&["unit"])),
            field("every", Ty::Duration),
        ],
    },
    Block {
        name: "channel",
        fields: &[
            field(
                "form",
                Ty::Enum(&["direct", "broadcast", "queue", "record"]),
            ),
            field(
                "audience",
                Ty::Enum(&["restricted", "internal", "external"]),
            ),
        ],
    },
    Block {
        name: "risk",
        fields: &[
            field("about", Ty::Ref(&[])),
            field("likelihood", Ty::Enum(LEVELS)),
            field("impact", Ty::Enum(LEVELS)),
            field(
                "treatment",
                Ty::Enum(&["accept", "reduce", "transfer", "avoid"]),
            ),
        ],
    },
    Block {
        name: "obligation",
        fields: &[
            field("source", Ty::Enum(&["law", "contract", "standard"])),
            field("about", Ty::Ref(&[])),
        ],
    },
];

/// Every block of the specification: what a KPI may belong to and what an extension may extend.
/// It is the list above and the blocks that are no part of a design.
pub const SPECIFIED: &[&str] = &[
    "shared",
    "party",
    "organization",
    "unit",
    "role",
    "position",
    "membership",
    "assignment",
    "competence",
    "decision",
    "decision_class",
    "delegation",
    "policy",
    "offering",
    "relationship",
    "goal",
    "asset",
    "budget",
    "process",
    "work",
    "agreement",
    "forum",
    "channel",
    "escalation",
    "responsibility",
    "risk",
    "obligation",
    "type",
    "design",
    "kpi",
    "experiment",
];

/// The blocks that hold a party or what one party is, has or holds. No KPI reads them.
pub const OVER_PARTY: &[&str] = &["party", "membership", "assignment", "competence"];

/// The blocks whose parts need one accountable position.
pub const SUBJECTS: &[&str] = &[
    "offering",
    "asset",
    "goal",
    "process",
    "policy",
    "budget",
    "agreement",
];

/// What a link may carry in every design.
pub const LINKS: &[&str] = &[
    "reports_to",
    "answers_to",
    "accountable",
    "responsible",
    "consulted",
    "informed",
    "decides",
    "delegates",
    "settles",
    "demand",
    "serves",
    "hands_off",
    "escalates",
    "funds",
];

/// The keys of a link.
pub const LINK_KEYS: &[&str] = &[
    "from", "to", "carries", "scope", "class", "limit", "mode", "after", "via",
];

pub const MODES: &[&str] = &["sole", "advice", "consent", "majority", "unanimous"];

/// The fields of a part that place it inside another part, read by a `parts` rule with `per`.
pub const PLACING: &[&str] = &["in", "convenes", "for", "scope"];

pub fn block(name: &str) -> Option<&'static Block> {
    BLOCKS.iter().find(|b| b.name == name)
}

impl Block {
    pub fn field(&self, name: &str) -> Option<&'static Field> {
        self.fields.iter().find(|f| f.name == name)
    }
}

impl Ty {
    /// Whether a kind may fix a field of this type.
    pub fn fixable(&self) -> bool {
        !matches!(self, Ty::TextList | Ty::Steps)
    }

    /// Why `value` is no value of this type. A reference is only held to be a name here.
    pub fn fault(&self, value: &Node) -> Option<String> {
        let fine = match self {
            Ty::Text => value.as_str().is_some(),
            Ty::Scalar | Ty::Ref(_) => value.is_scalar(),
            Ty::Integer { min } => value.as_i64().is_some_and(|v| v >= *min),
            Ty::Number => value.as_f64().is_some_and(f64::is_finite),
            Ty::Share => value.as_f64().is_some_and(|v| (0.0..=1.0).contains(&v)),
            Ty::Boolean => value.as_bool().is_some(),
            Ty::Duration => value.as_str().and_then(duration_seconds).is_some(),
            Ty::Enum(words) => value.as_str().is_some_and(|v| words.contains(&v)),
            Ty::TextList => {
                matches!(value, Node::Seq(items) if items.iter().all(|i| i.as_str().is_some()))
            }
            Ty::Steps => {
                matches!(value, Node::Seq(items) if items.iter().all(|i| matches!(i, Node::Map(_))))
            }
        };
        if fine {
            return None;
        }
        Some(match self {
            Ty::Text => "is not a string".to_string(),
            Ty::Scalar => "is not a single value".to_string(),
            Ty::Ref(_) => "is not the name of a part".to_string(),
            Ty::Integer { min } => format!("is not an integer of at least {min}"),
            Ty::Number => "is not a number".to_string(),
            Ty::Share => "is not a number from 0 to 1".to_string(),
            Ty::Boolean => "is neither true nor false".to_string(),
            Ty::Duration => not_a_duration().to_string(),
            Ty::Enum(words) => format!("is none of {}", words.join(", ")),
            Ty::TextList => "is not a list of strings".to_string(),
            Ty::Steps => "is not a list of steps".to_string(),
        })
    }
}

pub fn not_a_duration() -> &'static str {
    "is not a duration such as 90s, 15m, 24h, 7d or 2w"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_part_block_is_a_specified_block() {
        for b in BLOCKS {
            assert!(SPECIFIED.contains(&b.name), "{}", b.name);
            for f in b.fields {
                assert!(!COMMON.contains(&f.name), "{}.{}", b.name, f.name);
                if let Ty::Ref(blocks) = &f.ty {
                    for name in *blocks {
                        assert!(block(name).is_some(), "{}.{} names {name}", b.name, f.name);
                    }
                }
            }
        }
        assert_eq!(BLOCKS.len(), 17);
        for name in SUBJECTS {
            assert!(block(name).is_some(), "{name}");
        }
    }

    #[test]
    fn a_value_is_held_to_its_type() {
        assert!(Ty::Integer { min: 1 }.fault(&Node::Int(1)).is_none());
        assert!(Ty::Integer { min: 1 }.fault(&Node::Int(0)).is_some());
        assert!(Ty::Integer { min: 1 }.fault(&Node::Float(1.5)).is_some());
        assert!(Ty::Share.fault(&Node::Float(0.75)).is_none());
        assert!(Ty::Share.fault(&Node::Float(1.5)).is_some());
        assert!(Ty::Duration.fault(&Node::Str("7d".into())).is_none());
        assert!(Ty::Duration.fault(&Node::Str("weekly".into())).is_some());
        assert!(
            Ty::Enum(LEVELS)
                .fault(&Node::Str("severe".into()))
                .is_some()
        );
    }
}
