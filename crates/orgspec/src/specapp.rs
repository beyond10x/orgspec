//! `orgspec specapp`: the checker answering specapp's tool protocol.
//!
//! specapp (github.com/beyond10x/specapp) runs `orgspec specapp <verb> [arguments]` in the
//! repository directory and reads one JSON document from stdout:
//!
//! - `describe`: `specapp-tool/1`, the verbs and the format each verb answers in;
//! - `check <document>`: `specapp-findings/1`, `valid` and one finding per violation, each
//!   naming its rule, whether the rule is a gate, its subject (`part:<name>`,
//!   `link:<from>-><to>:<carries>` or `document`) and a sentence.
//!
//! Exit 0 with the document; a design with violations is exit 0 with `valid: false`. Exit 2
//! with `{"error": ...}` when the verb is unknown or the document cannot be read.

use crate::check::DesignReport;
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const TOOL_FORMAT: &str = "specapp-tool/1";
pub const FINDINGS_FORMAT: &str = "specapp-findings/1";

/// The answer to `describe`.
pub fn describe() -> Value {
    json!({
        "format": TOOL_FORMAT,
        "verbs": ["describe", "check"],
        "formats": { "check": FINDINGS_FORMAT },
        "edit_verbs": [],
        "kpis": [],
    })
}

/// The subject of a violation in specapp's words, and the message: a part of the design, a
/// link the design wrote or a default added, else the whole document. A violation whose
/// subject is neither a part nor a link (a name the border lists, the organization) keeps that
/// name at the start of its message.
pub fn subject(report: &DesignReport, name: &str, message: &str) -> (String, String) {
    if report.part_names.iter().any(|p| p == name) {
        (format!("part:{name}"), message.to_string())
    } else if report.link_names.iter().any(|l| l == name) {
        (format!("link:{name}"), message.to_string())
    } else if name == report.organization {
        (String::from("document"), message.to_string())
    } else {
        (String::from("document"), format!("{name}: {message}"))
    }
}

/// The answer to `check` for a design that was read: its violations, or the refusals of its
/// types (the design is not checked then, and each refusal is a gated finding about the whole
/// document).
pub fn findings(report: &DesignReport) -> Value {
    let mut findings: Vec<Value> = Vec::new();
    for r in &report.refusals {
        findings.push(json!({
            "rule": r.refusal,
            "gate": true,
            "subject": "document",
            "message": format!("{}: {}", r.subject, r.message),
        }));
    }
    for v in &report.violations {
        let (subject, message) = subject(report, &v.subject, &v.message);
        findings.push(json!({
            "rule": v.rule,
            "gate": v.gate,
            "subject": subject,
            "message": message,
        }));
    }
    let mut counts: BTreeMap<String, usize> = report.kpis.clone();
    for r in &report.refusals {
        *counts.entry(r.refusal.to_string()).or_default() += 1;
    }
    json!({
        "format": FINDINGS_FORMAT,
        "valid": report.valid,
        "findings": findings,
        "counts": counts,
    })
}

/// An answer of exit 2.
pub fn error(message: impl Into<String>) -> Value {
    json!({ "error": message.into() })
}
