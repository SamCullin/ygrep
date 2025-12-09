# ygrep

A fast, local, indexed code search tool optimized for AI coding assistants. Written in Rust using Tantivy for full-text indexing.

## Features

- **Literal text matching** - Works like grep, special characters included (`{% block`, `->get(`, etc.)
- **Fast indexed search** - Tantivy-powered BM25 ranking
- **Optional semantic search** - HNSW vector index with local embeddings (bge-small-en-v1.5)
- **Symlink handling** - Follows symlinks with cycle detection
- **AI-optimized output** - Clean, parseable results for LLM tools

## Installation

```bash
cargo install --path crates/ygrep-cli
```

## Usage

```bash
# Index a directory
ygrep index
ygrep index --rebuild              # Force rebuild
ygrep index --embeddings           # Include semantic embeddings (slower)

# Search (literal text matching)
ygrep search "{% block"
ygrep search "->get("
ygrep search "authentication"

# With options
ygrep search "error" -n 20         # Limit results
ygrep search "config" -e rs -e toml # Filter by extension

# Check status
ygrep status
```

## Example Output

```
# 10 results (8.0ms)

1. `src/config.rs:45-67`
```rust
pub struct Config {
    pub data_dir: PathBuf,
    pub max_file_size: u64,
}
```

2. `src/main.rs:12-28`
...
```

## How It Works

1. **Indexing**: Walks directory tree, indexes text files with Tantivy
2. **Search**: Extracts searchable terms, queries index, post-filters for exact literal match
3. **Results**: Returns files containing the exact query string (case-insensitive)

## Configuration

Index data stored in:
- macOS: `~/Library/Application Support/ygrep/indexes/`
- Linux: `~/.local/share/ygrep/indexes/`

## License

MIT
