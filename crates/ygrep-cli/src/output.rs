//! Output formatting utilities
//!
//! Most formatting is done in ygrep-core's SearchResult type.
//! This module provides additional CLI-specific formatting if needed.

use std::collections::HashMap;
use std::path::Path;

use ygrep_core::search::SearchHit;

const DEFAULT_BAR_WIDTH: usize = 20;

#[derive(Debug, Default)]
struct TreeNode {
    name: String,
    count: usize,
    truncated: bool,
    children: HashMap<String, TreeNode>,
}

impl TreeNode {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            count: 0,
            truncated: false,
            children: HashMap::new(),
        }
    }

    fn add_hit(&mut self, segments: &[&str], depth: usize) {
        self.count += 1;
        if depth == 0 {
            if !segments.is_empty() {
                self.truncated = true;
            }
            return;
        }
        if segments.is_empty() {
            return;
        }

        let segment = segments[0];
        let child = self
            .children
            .entry(segment.to_string())
            .or_insert_with(|| TreeNode::new(segment));
        child.add_hit(&segments[1..], depth - 1);
    }

    fn max_count(&self) -> usize {
        let mut max_count = self.count;
        for child in self.children.values() {
            max_count = max_count.max(child.max_count());
        }
        max_count
    }
}

pub fn format_tree_heatmap(hits: &[SearchHit], depth: Option<usize>) -> String {
    if hits.is_empty() {
        return "# 0 hits\n".to_string();
    }

    let mut root = TreeNode::new("");

    for hit in hits {
        let segments: Vec<String> = Path::new(&hit.path)
            .components()
            .filter_map(|component| component.as_os_str().to_str().map(|s| s.to_string()))
            .filter(|segment| !segment.is_empty())
            .collect();
        if segments.is_empty() {
            continue;
        }
        let depth_limit = depth.unwrap_or(segments.len()).max(1);
        let segment_refs: Vec<&str> = segments.iter().map(|s| s.as_str()).collect();
        root.add_hit(&segment_refs, depth_limit);
    }

    let max_count = root
        .children
        .values()
        .map(TreeNode::max_count)
        .max()
        .unwrap_or(0);
    let count_width = max_count.max(1).to_string().len();

    let mut output = String::new();
    output.push_str(&format!("# {} hits\n\n", hits.len()));

    let mut children: Vec<&TreeNode> = root.children.values().collect();
    children.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
    let label_width = max_tree_label_width(&children, "", true);
    for (index, child) in children.iter().enumerate() {
        let is_last = index + 1 == children.len();
        format_tree_node(
            child,
            "",
            true,
            is_last,
            max_count,
            count_width,
            label_width,
            &mut output,
        );
    }

    output
}

fn format_tree_node(
    node: &TreeNode,
    prefix: &str,
    use_connector: bool,
    is_last: bool,
    max_count: usize,
    count_width: usize,
    label_width: usize,
    output: &mut String,
) {
    let label = node_label(node);
    let connector = if use_connector {
        if is_last {
            "`- "
        } else {
            "|- "
        }
    } else {
        ""
    };
    let line = format!("{}{}{}", prefix, connector, label);
    let padding = label_width.saturating_sub(line.len());
    let count_str = format!("{:>width$}", node.count, width = count_width);
    let bar = render_bar(node.count, max_count);
    if bar.is_empty() {
        output.push_str(&format!("{}{}  {}\n", line, " ".repeat(padding), count_str));
    } else {
        output.push_str(&format!(
            "{}{}  {} {}\n",
            line,
            " ".repeat(padding),
            count_str,
            bar
        ));
    }

    let mut children: Vec<&TreeNode> = node.children.values().collect();
    children.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
    if children.is_empty() {
        return;
    }

    let branch = if use_connector {
        if is_last {
            "   "
        } else {
            "|  "
        }
    } else {
        ""
    };
    let child_prefix = format!("{}{}", prefix, branch);
    for (index, child) in children.iter().enumerate() {
        let child_is_last = index + 1 == children.len();
        format_tree_node(
            child,
            &child_prefix,
            true,
            child_is_last,
            max_count,
            count_width,
            label_width,
            output,
        );
    }
}

fn node_label(node: &TreeNode) -> String {
    if node.children.is_empty() && !node.truncated {
        node.name.clone()
    } else if node.truncated && node.children.is_empty() {
        format!("{}/...", node.name)
    } else {
        format!("{}/", node.name)
    }
}

fn max_tree_label_width(nodes: &[&TreeNode], prefix: &str, use_connector: bool) -> usize {
    let mut max_width = 0;
    for (index, node) in nodes.iter().enumerate() {
        let is_last = index + 1 == nodes.len();
        let connector = if use_connector {
            if is_last {
                "`- "
            } else {
                "|- "
            }
        } else {
            ""
        };
        let label = node_label(node);
        let line_len = prefix.len() + connector.len() + label.len();
        max_width = max_width.max(line_len);

        let mut children: Vec<&TreeNode> = node.children.values().collect();
        children.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
        if children.is_empty() {
            continue;
        }

        let branch = if use_connector {
            if is_last {
                "   "
            } else {
                "|  "
            }
        } else {
            ""
        };
        let child_prefix = format!("{}{}", prefix, branch);
        max_width = max_width.max(max_tree_label_width(&children, &child_prefix, true));
    }
    max_width
}

fn render_bar(count: usize, max_count: usize) -> String {
    if max_count == 0 {
        return String::new();
    }
    let ratio = count as f64 / max_count as f64;
    let mut bar_len = (ratio * DEFAULT_BAR_WIDTH as f64).round() as usize;
    if count > 0 && bar_len == 0 {
        bar_len = 1;
    }
    "#".repeat(bar_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ygrep_core::search::{MatchType, SearchHit};

    fn make_hit(path: &str) -> SearchHit {
        SearchHit {
            path: path.to_string(),
            line_start: 1,
            line_end: 1,
            snippet: "example".to_string(),
            score: 0.5,
            is_chunk: false,
            doc_id: path.to_string(),
            match_type: MatchType::Text,
        }
    }

    #[test]
    fn formats_tree_with_depth_cutoff() {
        let hits = vec![
            make_hit("src/api/auth.rs"),
            make_hit("src/api/users.rs"),
            make_hit("tests/auth.rs"),
        ];

        let output = format_tree_heatmap(&hits, Some(2));

        assert!(output.contains("# 3 hits"));
        let src_line = output
            .lines()
            .find(|line| line.contains("src/"))
            .unwrap_or("");
        let api_line = output
            .lines()
            .find(|line| line.contains("api/"))
            .unwrap_or("");
        let tests_line = output
            .lines()
            .find(|line| line.contains("tests/"))
            .unwrap_or("");
        assert!(src_line.contains(" 2 "));
        assert!(api_line.contains(" 2 "));
        assert!(tests_line.contains(" 1 "));
    }
}
