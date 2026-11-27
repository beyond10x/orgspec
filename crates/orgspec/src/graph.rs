//! Reachability, cycles and the longest chain in a small graph of names.

use std::collections::{BTreeMap, BTreeSet};

/// For each name, the names its edges lead to. Every name of the graph is a key.
pub type Graph = BTreeMap<String, Vec<String>>;

pub fn graph<'a>(edges: impl IntoIterator<Item = (&'a str, &'a str)>) -> Graph {
    let mut graph = Graph::new();
    for (from, to) in edges {
        graph.entry(to.to_string()).or_default();
        graph
            .entry(from.to_string())
            .or_default()
            .push(to.to_string());
    }
    graph
}

/// Every name reached from `start` over one edge or more.
pub fn reach(graph: &Graph, start: &str) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut stack: Vec<&String> = graph.get(start).into_iter().flatten().collect();
    while let Some(name) = stack.pop() {
        if seen.insert(name.clone()) {
            stack.extend(graph.get(name).into_iter().flatten());
        }
    }
    seen
}

/// The groups of names that lie on a common cycle. Each group is sorted, and the groups are
/// sorted by their first name.
pub fn cycles(graph: &Graph) -> Vec<Vec<String>> {
    let reached: BTreeMap<String, BTreeSet<String>> = graph
        .keys()
        .map(|name| (name.clone(), reach(graph, name)))
        .collect();
    let mut groups: Vec<Vec<String>> = Vec::new();
    for (name, from_name) in &reached {
        if !from_name.contains(name) || groups.iter().any(|g| g.contains(name)) {
            continue;
        }
        groups.push(
            from_name
                .iter()
                .filter(|other| reached[*other].contains(name))
                .cloned()
                .collect(),
        );
    }
    groups
}

/// The number of edges of the longest chain that starts at `from` and visits no name twice.
pub fn longest(graph: &Graph, from: &str) -> usize {
    fn walk<'a>(graph: &'a Graph, at: &'a str, path: &mut Vec<&'a str>) -> usize {
        path.push(at);
        let mut best = 0;
        for next in graph.get(at).into_iter().flatten() {
            if !path.contains(&next.as_str()) {
                best = best.max(1 + walk(graph, next, path));
            }
        }
        path.pop();
        best
    }
    walk(graph, from, &mut Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycles_are_grouped_and_chains_measured() {
        let g = graph([("a", "b"), ("b", "c"), ("c", "b"), ("d", "d"), ("e", "a")]);
        assert_eq!(
            cycles(&g),
            vec![
                vec!["b".to_string(), "c".to_string()],
                vec!["d".to_string()]
            ]
        );
        assert_eq!(longest(&g, "e"), 3);
        assert_eq!(longest(&g, "d"), 0);
        assert!(reach(&g, "e").contains("c"));
        assert!(!reach(&g, "a").contains("a"));
    }
}
