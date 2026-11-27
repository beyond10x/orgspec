//! The command line: exit codes, the lines, the JSON output and the type path.

use super::common::fixtures;
use orgspec::structure;
use serde_json::Value;
use std::process::{Command, Output};

/// Runs `orgspec` in the fixture directory, where no `types` directory exists, without a type
/// path in the environment.
fn orgspec(args: &[&str]) -> Output {
    orgspec_in("", &[], args)
}

fn orgspec_in(dir: &str, env: &[(&str, &str)], args: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_orgspec"));
    command
        .current_dir(fixtures().join(dir))
        .env_remove("ORGSPEC_TYPE_PATH")
        .args(args);
    for (key, value) in env {
        command.env(key, value);
    }
    command.output().expect("orgspec runs")
}

fn stdout(output: &Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("JSON on stdout")
}

#[test]
fn check_exits_0_and_prints_the_summary_line() {
    let out = orgspec(&["check", "structure/designs/valid.yaml"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        [
            "orgspec check: structure/designs/valid.yaml: type shop, 25 parts, 30 links, 0 violation(s) in 0 gate(s), 0 not gated"
        ]
    );
}

#[test]
fn check_exits_1_and_prints_one_line_per_violation_in_order() {
    let out = orgspec(&["check", "structure/designs/bad_value.yaml"]);
    assert_eq!(out.status.code(), Some(1));
    let lines = stdout(&out);
    assert_eq!(lines.len(), 8, "{lines:#?}");
    assert_eq!(
        lines[0],
        "bad_value bakery `form: bazaar` is none of company, public_body, association, cooperative, partnership, project, informal"
    );
    assert!(lines[1].starts_with("bad_value head `"), "{lines:#?}");
    assert!(
        lines[4].starts_with("bad_value head->board:reports_to `"),
        "{lines:#?}"
    );
    assert_eq!(
        lines[7],
        "orgspec check: structure/designs/bad_value.yaml: type base, 5 parts, 1 links, 7 violation(s) in 1 gate(s), 0 not gated"
    );
}

#[test]
fn check_marks_a_violation_of_a_rule_that_is_no_gate() {
    let out = orgspec(&["check", "composition/designs/big-ward.yaml"]);
    assert_eq!(out.status.code(), Some(0));
    let lines = stdout(&out);
    assert_eq!(
        lines[0],
        "ward_is_small east `beds: 40` is above max 30 (not gated)"
    );
    assert!(
        lines[1].ends_with("0 violation(s) in 0 gate(s), 1 not gated"),
        "{lines:#?}"
    );
}

#[test]
fn check_of_several_files_exits_with_the_worst_code() {
    let valid = "structure/designs/valid.yaml";
    let broken = "structure/designs/unknown_role.yaml";
    let unreadable = "unreadable/missing-key.yaml";
    let out = orgspec(&["check", valid, broken]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(stdout(&out).len(), 3);
    let out = orgspec(&["check", unreadable, broken, valid]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(stdout(&out).len(), 3);
}

#[test]
fn check_exits_2_when_a_document_cannot_be_read() {
    for (file, reason) in [
        ("unreadable/not-yaml.yaml", "not YAML"),
        (
            "unreadable/unknown-format.yaml",
            "unknown format `orgspec-organization/9`",
        ),
        ("unreadable/missing-key.yaml", "missing key `parts`"),
        ("unreadable/absent.yaml", "cannot be read"),
        (
            "structure/types/shop.yaml",
            "unknown format `orgspec-type/1`",
        ),
    ] {
        let out = orgspec(&["check", file]);
        assert_eq!(out.status.code(), Some(2), "{file}");
        assert!(stdout(&out).is_empty(), "{file}");
        let error = stderr(&out);
        assert!(error.starts_with(&format!("orgspec: {file}: ")), "{error}");
        assert!(error.contains(reason), "{error}");
    }
}

#[test]
fn check_stops_with_1_when_a_type_of_the_design_is_refused() {
    let out = orgspec(&["check", "refused/designs/library.yaml"]);
    assert_eq!(out.status.code(), Some(1));
    let lines = stdout(&out);
    assert_eq!(lines.len(), 2, "{lines:#?}");
    assert!(
        lines[0].starts_with("unknown_carries hasty.asset_has_curator "),
        "{lines:#?}"
    );
    assert_eq!(
        lines[1],
        "orgspec check: refused/designs/library.yaml: type hasty refused, 1 refusal(s)"
    );
}

#[test]
fn check_json_prints_one_object_per_file() {
    let out = orgspec(&[
        "check",
        "--json",
        "rules/designs/default-holds.yaml",
        "composition/designs/big-ward-strict.yaml",
        "unreadable/missing-key.yaml",
    ]);
    assert_eq!(out.status.code(), Some(2));
    let all = json(&out);
    let all = all.as_array().expect("an array");
    assert_eq!(all.len(), 3);

    let first = &all[0];
    let keys: Vec<&str> = first
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        [
            "defaults",
            "file",
            "kpis",
            "links",
            "measures",
            "organization",
            "parts",
            "refusals",
            "types",
            "valid",
            "violations"
        ]
    );
    assert_eq!(first["file"], "rules/designs/default-holds.yaml");
    assert_eq!(first["organization"], "library");
    assert_eq!(first["types"], serde_json::json!(["base", "stewarded"]));
    assert_eq!((&first["parts"], &first["links"]), (&11.into(), &8.into()));
    assert_eq!(first["valid"], true);
    assert_eq!(first["violations"], serde_json::json!([]));
    assert_eq!(
        first["defaults"],
        serde_json::json!({
            "steward_answers": ["sorter->head:answers_to", "fetcher->warden:answers_to"]
        })
    );
    let kpis = first["kpis"].as_object().unwrap();
    assert_eq!(kpis.len(), structure::RULES.len());
    assert!(kpis.values().all(|count| count == 0));
    assert_eq!(
        first["measures"],
        serde_json::json!({
            "positions": 5,
            "seats": 5,
            "agent_seats": 3.0,
            "units": 3,
            "span_max": 2,
            "reporting_depth_max": 2,
            "subjects": 0,
            "subjects_with_accountable": 0
        })
    );

    let second = &all[1];
    assert_eq!(second["valid"], false);
    assert_eq!(
        second["violations"],
        serde_json::json!([{
            "rule": "ward_is_small",
            "gate": true,
            "subject": "east",
            "message": "`beds: 40` is above max 30"
        }])
    );
    let kpis = second["kpis"].as_object().unwrap();
    assert_eq!(kpis.len(), structure::RULES.len() + 2);
    assert_eq!(kpis["ward_is_small"], 1);
    assert_eq!(kpis["care_unit_has_one_lead"], 0);

    let third = &all[2];
    assert_eq!(third["file"], "unreadable/missing-key.yaml");
    assert_eq!(third["valid"], false);
    assert_eq!(third["error"], "missing key `parts`");
}

#[test]
fn check_finds_a_type_through_the_flag_the_environment_and_the_working_directory() {
    let design = "typepath/far/design.yaml";
    let without = orgspec(&["check", design]);
    assert_eq!(without.status.code(), Some(1));
    let lines = stdout(&without);
    assert!(
        lines[0].starts_with("unknown_type lending the design `library` names it"),
        "{lines:#?}"
    );

    let flag = orgspec(&["check", "--type-path", "typepath/shelf", design]);
    assert_eq!(flag.status.code(), Some(0), "{:#?}", stdout(&flag));
    assert!(stdout(&flag)[0].contains("type lending, 5 parts, 1 links"));

    let shelf = fixtures().join("typepath/shelf");
    let path = format!("{}:", shelf.display());
    let env = orgspec_in("", &[("ORGSPEC_TYPE_PATH", &path)], &["check", design]);
    assert_eq!(env.status.code(), Some(0), "{:#?}", stdout(&env));

    let cwd = orgspec_in("typepath/cwd", &[], &["check", "../far/design.yaml"]);
    assert_eq!(cwd.status.code(), Some(0), "{:#?}", stdout(&cwd));
}

#[test]
fn a_directory_named_twice_is_read_once_and_a_type_in_two_files_is_refused() {
    let design = "typepath/far/design.yaml";
    let shelf = fixtures().join("typepath/shelf");
    let again = shelf.display().to_string();
    let twice = orgspec(&[
        "check",
        "--type-path",
        "typepath/shelf",
        "--type-path",
        "typepath/far/../shelf",
        "--type-path",
        &again,
        design,
    ]);
    assert_eq!(twice.status.code(), Some(0), "{:#?}", stdout(&twice));

    let two_files = orgspec(&[
        "check",
        "--type-path",
        "typepath/shelf",
        "--type-path",
        "typepath/cwd/types",
        design,
    ]);
    assert_eq!(two_files.status.code(), Some(1));
    let lines = stdout(&two_files);
    assert!(
        lines[0].starts_with("type_declared_twice lending declared by "),
        "{lines:#?}"
    );

    let missing = orgspec(&["check", "--type-path", "typepath/nowhere", design]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(stderr(&missing).contains("type path typepath/nowhere"));
}

#[test]
fn type_check_exits_0_1_and_2() {
    let accepted = orgspec(&["type", "check", "accepted/full.yaml", "accepted/lower.yaml"]);
    assert_eq!(accepted.status.code(), Some(0), "{:#?}", stdout(&accepted));
    assert_eq!(
        stdout(&accepted),
        [
            "orgspec type check: accepted/full.yaml: type full, set base+lower+full, 2 kinds, 1 links, 4 rules, 1 defaults, 2 kpis, 0 refusals",
            "orgspec type check: accepted/lower.yaml: type lower, set base+lower, 2 kinds, 0 links, 1 rules, 0 defaults, 0 kpis, 0 refusals",
        ]
    );

    let refused = orgspec(&["type", "check", "refusals/gate_loosened/subject.yaml"]);
    assert_eq!(refused.status.code(), Some(1));
    let lines = stdout(&refused);
    assert_eq!(lines.len(), 3, "{lines:#?}");
    assert!(
        lines[0].starts_with("gate_loosened upper.one_board "),
        "{lines:#?}"
    );
    assert_eq!(
        lines[2],
        "orgspec type check: refusals/gate_loosened/subject.yaml: type upper refused, 2 refusal(s)"
    );

    for file in [
        "unreadable/type-missing-key.yaml",
        "unreadable/type-unknown-format.yaml",
        "unreadable/not-yaml.yaml",
        "structure/designs/valid.yaml",
    ] {
        let unreadable = orgspec(&["type", "check", file]);
        assert_eq!(unreadable.status.code(), Some(2), "{file}");
        assert!(stderr(&unreadable).starts_with(&format!("orgspec: {file}: ")));
    }
}

#[test]
fn type_check_finds_an_extended_type_through_the_flag() {
    let alone = orgspec(&["type", "check", "composition/types/strict-clinic.yaml"]);
    assert_eq!(alone.status.code(), Some(0));
    let far = orgspec_in(
        "refusals/unknown_type",
        &[],
        &["type", "check", "subject.yaml"],
    );
    assert_eq!(far.status.code(), Some(1));
    assert!(stdout(&far)[0].starts_with("unknown_type gallery type `museum` names it"));
}

#[test]
fn type_check_json_prints_the_refusals() {
    let out = orgspec(&[
        "type",
        "check",
        "--json",
        "refusals/kpi_over_party/subject.yaml",
    ]);
    assert_eq!(out.status.code(), Some(1));
    let all = json(&out);
    let one = &all[0];
    assert_eq!(one["type"], "museum");
    assert_eq!(one["valid"], false);
    assert_eq!(one["types"], serde_json::json!(["base", "museum"]));
    assert_eq!(one["refusals"].as_array().unwrap().len(), 2);
    assert_eq!(one["refusals"][0]["refusal"], "kpi_over_party");
    assert_eq!(one["refusals"][0]["subject"], "museum.hours_per_holder");
}

#[test]
fn a_command_without_a_file_is_a_usage_error() {
    assert_eq!(orgspec(&["check"]).status.code(), Some(2));
    assert_eq!(orgspec(&["type", "check"]).status.code(), Some(2));
    assert_eq!(orgspec(&["--version"]).status.code(), Some(0));
}
