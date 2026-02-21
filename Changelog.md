# Changelog

All notable changes to this project are documented in this file.

## [v2-beta5] - 2026-02-21

### Improved
- Stabilized `--rust-download` async range/chunk downloader path with `indicatif`-based multi progress output.
- Improved phase visibility and terminal UX for extract/download/merge flow.

## [v2-beta4] - 2026-02-21

### Improved
- Parallelized split stream downloads (video/audio) in `--rust-download` mode for better speed.
- Added terminal flow stages and per-stream progress percentage output.

## [v2-beta3] - 2026-02-21

### Added
- Added split stream handling in `--rust-download` mode to download video/audio streams separately and merge with `ffmpeg -c copy`.

### Improved
- Added richer stream diagnostics (`format_id`, codec info) to error logs for investigation.

## [v2-beta2] - 2026-02-21

### Fixed
- Improved `--rust-download` extraction parsing to pick direct media URLs from `requested_formats` / `formats` when `requested_downloads` is missing.
- Improved extraction format selection to reduce false negatives for single URL Rust download mode.

## [v2-beta1] - 2026-02-21

### Added
- Added experimental `--rust-download` mode for single URL operation.
- Added detailed investigation logs under `%USERPROFILE%/downloader/errorlog/*.log`.

### Changed
- In `--rust-download` mode, yt-dlp is used for extraction only (`-J`) and actual download is handled by Rust.
- No automatic fallback in this mode; rerun without `--rust-download` when hang/failure occurs.

## [1.3.3] - 2026-02-21

### Added
- Added `-t, --threads <INT>` option for batch mode (`--urls`) to control maximum worker threads.

### Changed
- Batch mode now executes downloads in parallel threads while suppressing raw yt-dlp logs in terminal output.
- Worker count now safely falls back to URL count when requested threads exceed the number of input URLs.

## [1.3.2] - 2026-02-21

### Added
- Added custom platform detection for niconico (`nicovideo.jp`, `nico.ms`), SoundCloud (`soundcloud.com`), Instagram (`instagram.com`), TikTok (`tiktok.com`), and bilibili (`bilibili.com`, `b23.tv`).

### Changed
- Added per-platform yt-dlp defaults for the newly supported platforms.
- SoundCloud now skips `--merge-output-format` to better match audio-first downloads.

### Docs
- Updated `README.md` and `japanese/README_ja.md` platform support sections.

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
