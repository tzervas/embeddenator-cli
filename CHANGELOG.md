# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.21.2] - 2026-01-26

### Changed
- **Supply Chain Security**: Documented maintained dependency ecosystem for unmaintained crates
  - See [MAINTAINED_DEPENDENCIES.md](../MAINTAINED_DEPENDENCIES.md) for integration guide
  - Upstream PR: https://github.com/huggingface/candle/pull/3335

## [0.21.2] - 2026-01-26

### Changed
- **Supply Chain Security**: Documented maintained dependency ecosystem for ML crates
  - See [MAINTAINED_DEPENDENCIES.md](../MAINTAINED_DEPENDENCIES.md) for maintained forks of unmaintained `paste` and `gemm` crates
  - Upstream PR to huggingface/candle: https://github.com/huggingface/candle/pull/3335

## [0.21.1] - 2026-01-25

### Added
- Incremental update commands now functional: add, remove, modify, compact
- Wired CLI commands to EmbrFS methods in embeddenator-fs

### Changed
- Updated dependencies to stable versions

## [0.21.0] - 2026-01-25

### Changed
- Updated dependencies to stable versions

### Fixed
- Correct feature propagation for FUSE support

## [0.20.0-alpha.1] - 2026-01-16

### Added
- Initial alpha release
- CLI commands for embeddenator operations
- Integration with embeddenator-vsa, embeddenator-retrieval, embeddenator-fs, embeddenator-io
