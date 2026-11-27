//! `orgspec specapp`: the checker answers specapp's tool protocol. `describe` names the verbs;
//! `check <design>` maps every violation to a finding whose subject is a part, a link or the
//! document; exit 2 carries `{error}`.

use super::common::{design, fixtures, yaml_files};
use orgspec::specapp;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::{Command, Output};

/// The repository root, where specapp runs the tool.
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn tool(dir: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_orgspec"))
        .current_dir(dir)
        .env_remove("ORGSPEC_TYPE_PATH")
        .arg("specapp")
        .args(args)
        .output()
        .expect("orgspec runs")
}

fn json(out: &Output) -> Value {
    serde_json::from_slice(&out.stdout).expect("one JSON document on stdout")
}

/// The shape of `specapp-findings/1`.
fn assert_findings(value: &Value) {
    assert_eq!(value["format"], "specapp-findings/1");
    assert!(value["valid"].is_boolean());
    for f in value["findings"].as_array().expect("findings") {
        assert!(!f["rule"].as_str().unwrap().is_empty(), "{f}");
        assert!(f["gate"].is_boolean(), "{f}");
        assert!(f["message"].is_string(), "{f}");
        let subject = f["subject"].as_str().unwrap();
        assert!(
            subject == "document"
                || subject.len() > 5
                    && (subject.starts_with("part:") || subject.starts_with("link:")),
            "{subject}"
        );
    }
}

#[test]
fn describe_names_check_and_its_format() {
    let out = tool(&root(), &["describe"]);
    assert_eq!(out.status.code(), Some(0));
    let value = json(&out);
    assert_eq!(value["format"], "specapp-tool/1");
    assert_eq!(value["verbs"], serde_json::json!(["describe", "check"]));
    assert_eq!(value["formats"]["check"], "specapp-findings/1");
}

#[test]
fn check_of_a_valid_design_is_valid_with_no_gated_finding() {
    let out = tool(&fixtures(), &["check", "structure/designs/valid.yaml"]);
    assert_eq!(out.status.code(), Some(0));
    let value = json(&out);
    assert_findings(&value);
    assert_eq!(value["valid"], true);
    assert_eq!(value["findings"], serde_json::json!([]));
}

#[test]
fn every_broken_design_answers_its_rule_on_a_part_a_link_or_the_document() {
    let dir = root();
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir.join("organizations/broken"))
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "yaml"))
        .collect();
    files.sort();
    assert!(!files.is_empty());
    for file in &files {
        let rel = file
            .strip_prefix(&dir)
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let out = tool(&dir, &["check", &rel]);
        assert_eq!(out.status.code(), Some(0), "{rel}");
        let value = json(&out);
        assert_findings(&value);
        assert_eq!(value["valid"], false, "{rel}");
        let rule = file.file_stem().unwrap().to_string_lossy().into_owned();
        let gated: BTreeSet<&str> = value["findings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["gate"] == true)
            .map(|f| f["rule"].as_str().unwrap())
            .collect();
        assert_eq!(gated, BTreeSet::from([rule.as_str()]), "{rel}");
        assert_eq!(value["counts"][&rule], 1, "{rel}");
    }
}

#[test]
fn a_subject_is_a_part_a_written_link_or_the_document() {
    for path in yaml_files("structure/designs") {
        let report = design(&path);
        let value = specapp::findings(&report);
        assert_findings(&value);
        let findings = value["findings"].as_array().unwrap();
        assert_eq!(findings.len(), report.violations.len(), "{path}");
        for (f, v) in findings.iter().zip(&report.violations) {
            assert_eq!(f["rule"], v.rule.as_str(), "{path}");
            let subject = f["subject"].as_str().unwrap();
            if let Some(part) = subject.strip_prefix("part:") {
                assert!(report.part_names.iter().any(|p| p == part), "{path}");
            } else if let Some(link) = subject.strip_prefix("link:") {
                assert!(report.link_names.iter().any(|l| l == link), "{path}");
            } else if v.subject != report.organization {
                assert!(
                    f["message"]
                        .as_str()
                        .unwrap()
                        .starts_with(&format!("{}: ", v.subject)),
                    "{path}: the name stays in the message"
                );
            }
        }
    }
}

#[test]
fn a_link_violation_names_the_link_and_a_border_name_stays_in_the_message() {
    let report = design("structure/designs/unknown_carries.yaml");
    let value = specapp::findings(&report);
    assert!(
        value["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["subject"].as_str().unwrap().starts_with("link:")),
        "{value}"
    );
    let report = design("structure/designs/border_without_part.yaml");
    let value = specapp::findings(&report);
    let f = &value["findings"][0];
    assert_eq!(f["subject"], "document");
    assert!(f["message"].as_str().unwrap().contains(": "), "{f}");
}

#[test]
fn an_unreadable_design_or_an_unknown_verb_is_exit_2_with_an_error() {
    for args in [
        vec!["check", "unreadable/not-yaml.yaml"],
        vec!["check", "no-such-file.yaml"],
        vec!["check"],
        vec!["check", "a.yaml", "b.yaml"],
        vec!["describe", "extra"],
        vec!["simulate", "a.yaml"],
        vec![],
    ] {
        let out = tool(&fixtures(), &args);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(json(&out)["error"].is_string(), "{args:?}");
    }
}
