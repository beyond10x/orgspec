//! The fixtures, and the two checks as the command runs them without flags.

use orgspec::check::{DesignReport, TypeReport, check_design, check_type};
use std::path::PathBuf;

pub fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Checks a design fixture with the types in `types` beside its directory, if there are any.
pub fn design(path: &str) -> DesignReport {
    let file = fixtures().join(path);
    let types = file.parent().expect("a directory").join("../types");
    let dirs: Vec<PathBuf> = types.is_dir().then_some(types).into_iter().collect();
    check_design(&file, &dirs).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// Checks a type fixture with the types of its own directory.
pub fn type_doc(path: &str) -> TypeReport {
    let file = fixtures().join(path);
    let dir = file.parent().expect("a directory").to_path_buf();
    check_type(&file, &[dir]).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The rule and the subject of every violation, in the order of the report.
pub fn violated(report: &DesignReport) -> Vec<(&str, &str)> {
    report
        .violations
        .iter()
        .map(|v| (v.rule.as_str(), v.subject.as_str()))
        .collect()
}

/// Every `*.yaml` of a fixture directory, as fixture paths, in order.
pub fn yaml_files(dir: &str) -> Vec<String> {
    let mut files: Vec<String> = std::fs::read_dir(fixtures().join(dir))
        .unwrap_or_else(|e| panic!("{dir}: {e}"))
        .map(|entry| entry.expect("an entry").file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".yaml"))
        .map(|name| format!("{dir}/{name}"))
        .collect();
    files.sort();
    files
}
