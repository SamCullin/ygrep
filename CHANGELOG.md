# Changelog

All notable changes to ygrep will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-12-09

### Changed
- Renamed `--embeddings` flag to `--semantic` for clarity
- Changed user-facing terminology from "embedding" to "semantic" throughout
- Progress bar now displays correctly (model loads before bar starts)

### Fixed
- Fixed line numbers in hybrid search showing top of file instead of match location
- Fixed `-n` limit not working with shorthand query form (`ygrep -n 5 query`)
- Fixed UTF-8 panic when displaying results with non-ASCII characters

## [0.2.5] - 2025-12-09

### Changed
- Optimized vector index loading (3+ seconds → ~5ms) using native HNSW dump/load
- Removed embedding daemon (no longer needed after optimization)

### Fixed
- Fixed UTF-8 character boundary panic in search results

## [0.2.4] - 2025-12-08

### Changed
- Various performance improvements and bug fixes

## [0.2.3] - 2025-12-07

### Changed
- Version bump for release

## [0.2.2] - 2025-12-06

### Added
- ONNX Runtime support for semantic embeddings
- Hybrid search combining BM25 and vector similarity
- `--embeddings` flag to build semantic index during indexing

## [0.2.1] - 2025-12-05

### Changed
- Switched to rustls for easier cross-platform builds
- Removed OpenSSL dependency

## [0.2.0] - 2025-12-04

### Changed
- Updated build configuration for multiple target platforms
- Added more build targets (macOS ARM64, Linux x86_64)

## [0.1.0] - 2025-12-03

### Added
- Initial release
- Tantivy-based full-text indexing with BM25 ranking
- Code-aware tokenizer preserving `$`, `@`, `#` as part of tokens
- Literal text matching (like grep, not regex)
- File watching for incremental index updates
- Symlink handling with cycle detection
- AI-optimized output format
- Index management commands (`indexes list`, `indexes clean`, `indexes remove`)
- Client integrations for Claude Code, OpenCode, Codex, and Factory Droid
- Cross-platform support (macOS, Linux)

### Fixed
- Fixed cross-platform debouncer type for Linux builds
- Fixed file watcher to follow symlinks correctly
- Deduplicated watch events for same file

[0.2.6]: https://github.com/yetidevworks/ygrep/compare/v0.2.5...v0.3.0
[0.2.5]: https://github.com/yetidevworks/ygrep/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/yetidevworks/ygrep/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/yetidevworks/ygrep/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/yetidevworks/ygrep/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/yetidevworks/ygrep/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/yetidevworks/ygrep/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/yetidevworks/ygrep/releases/tag/v0.1.0
