use anyhow::{Context, Result};
use std::path::Path;
use ygrep_core::search::{MatchType, SearchResult};
use ygrep_core::Workspace;

use crate::OutputFormat;

pub fn run(
    workspace_path: &Path,
    query: &str,
    limit: usize,
    extensions: Vec<String>,
    paths: Vec<String>,
    use_regex: bool,
    show_scores: bool,
    text_only: bool,
    format: OutputFormat,
) -> Result<()> {
    // Open existing workspace (fails if not indexed)
    let workspace = match Workspace::open(workspace_path) {
        Ok(ws) => ws,
        Err(_) => {
            eprintln!("Workspace not indexed: {}", workspace_path.display());
            eprintln!();
            eprintln!("To index this workspace, run:");
            eprintln!("  ygrep index              # Text-only (fast)");
            eprintln!("  ygrep index --semantic   # With semantic search (slower, better results)");
            std::process::exit(1);
        }
    };

    // Search: use hybrid search by default if semantic index is available
    #[cfg(feature = "embeddings")]
    let use_hybrid = !text_only && workspace.has_semantic_index();
    #[cfg(not(feature = "embeddings"))]
    let use_hybrid = false;
    let _ = text_only; // Suppress unused warning when embeddings disabled

    // Hold copies so we can consistently apply filters after search (hybrid ignores them)
    let extension_filters = extensions.clone();
    let path_filters = paths.clone();

    let mut result = if use_hybrid && !use_regex {
        // Hybrid search (BM25 + vector with RRF) - not supported with regex
        #[cfg(feature = "embeddings")]
        {
            workspace.search_hybrid(query, Some(limit))
                .context("Hybrid search failed")?
        }
        #[cfg(not(feature = "embeddings"))]
        unreachable!()
    } else {
        // Build filters for text-only search
        let ext_filter = if extensions.is_empty() { None } else { Some(extensions) };
        let path_filter = if paths.is_empty() { None } else { Some(paths) };

        workspace.search_filtered(query, Some(limit), ext_filter, path_filter, use_regex)
            .context("Search failed")?
    };

    // Apply filters to hybrid results (text search is a no-op)
    apply_filters(&mut result, &extension_filters, &path_filters);

    // Output results
    let output = match format {
        OutputFormat::Ai => result.format_ai(),
        OutputFormat::Json => result.format_json(),
        OutputFormat::Pretty => result.format_pretty(show_scores),
    };

    print!("{}", output);

    Ok(())
}

fn apply_filters(result: &mut SearchResult, extensions: &[String], paths: &[String]) {
    if extensions.is_empty() && paths.is_empty() {
        return;
    }

    if !extensions.is_empty() {
        result.hits.retain(|hit| {
            Path::new(&hit.path)
                .extension()
                .map(|ext| {
                    extensions
                        .iter()
                        .any(|allowed| allowed.eq_ignore_ascii_case(&ext.to_string_lossy()))
                })
                .unwrap_or(false)
        });
    }

    if !paths.is_empty() {
        result.hits.retain(|hit| {
            paths
                .iter()
                .any(|pattern| hit.path.starts_with(pattern) || hit.path.contains(pattern))
        });
    }

    result.total = result.hits.len();
    result.text_hits = result
        .hits
        .iter()
        .filter(|hit| matches!(hit.match_type, MatchType::Text | MatchType::Hybrid))
        .count();
    result.semantic_hits = result
        .hits
        .iter()
        .filter(|hit| matches!(hit.match_type, MatchType::Semantic | MatchType::Hybrid))
        .count();
}

#[cfg(test)]
mod tests {
    use super::*;
    use ygrep_core::search::{MatchType, SearchHit};

    fn make_hit(path: &str, match_type: MatchType) -> SearchHit {
        SearchHit {
            path: path.to_string(),
            line_start: 1,
            line_end: 1,
            snippet: "example".to_string(),
            score: 0.5,
            is_chunk: false,
            doc_id: path.to_string(),
            match_type,
        }
    }

    fn make_result(hits: Vec<SearchHit>) -> SearchResult {
        SearchResult {
            total: hits.len(),
            hits,
            query_time_ms: 0,
            text_hits: 0,
            semantic_hits: 0,
        }
    }

    #[test]
    fn filters_by_extension() {
        let mut result = make_result(vec![
            make_hit("src/main.rs", MatchType::Text),
            make_hit("src/lib.ts", MatchType::Semantic),
        ]);

        let extensions = vec!["rs".to_string()];
        apply_filters(&mut result, &extensions, &[]);

        assert_eq!(result.hits.len(), 1);
        assert_eq!(result.hits[0].path, "src/main.rs");
        assert_eq!(result.text_hits, 1);
        assert_eq!(result.semantic_hits, 0);
    }

    #[test]
    fn filters_by_path_pattern() {
        let mut result = make_result(vec![
            make_hit("src/main.rs", MatchType::Hybrid),
            make_hit("tests/test.rs", MatchType::Semantic),
        ]);

        let paths = vec!["tests".to_string()];
        apply_filters(&mut result, &[], &paths);

        assert_eq!(result.hits.len(), 1);
        assert_eq!(result.hits[0].path, "tests/test.rs");
        assert_eq!(result.semantic_hits, 1);
        assert_eq!(result.text_hits, 0);
    }
}
