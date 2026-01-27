# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.21.0] - 2026-01-27

### Changed
- CI: disable debug by default, selective retry on failure
- Code formatting and clippy fixes

### Fixed
- Fixed `too_many_arguments` clippy warning

## [0.20.0-alpha.2] - 2026-01-25

### Changed
- Updated dependencies to stable versions
- Fixed fuse feature flag configuration

### Fixed
- Correct feature propagation for FUSE support

## [0.20.0-alpha.1] - 2026-01-16

### Added
- Initial alpha release
- CLI commands for embeddenator operations
- Integration with embeddenator-vsa, embeddenator-retrieval, embeddenator-fs, embeddenator-io
