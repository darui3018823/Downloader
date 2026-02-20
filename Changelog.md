# Changelog

All notable changes to this project are documented in this file.

## [1.3.0] - 2026-02-20

### Changed
- Prioritized highest quality download format for Generic platform (`bv*+ba/b`).
- Subtitle behavior is now opt-in via CLI options (`--write-sub`, `--sub-lang`, `--sub-format`, `--convert-subs`).
- Moved release history from README files to this `Changelog.md`.

### Docs
- Simplified `README.md` and `japanese/README_ja.md` by removing version-specific history sections.
- Added changelog references in both README files.

## [1.2.0]

### Added
- Advanced CLI options for output directory, quality, format, metadata control, playlist mode, and logging.
- Cookie browser selection with Chrome default.
- Subtitle-related options and behavior controls.

### Improved
- Expanded CLI usability for interactive, single URL, and batch workflows.

## [1.1.0]

### Added
- CLI enhancements and improved argument handling.

## [1.0.0]

### Added
- Complete rewrite from Python implementation to Rust.
- Core downloader functionality powered by `yt-dlp`.
