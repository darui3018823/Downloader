# Video Downloader (Rust Version)

> 📖 **Languages:** [English](./README.md) | [日本語](./japanese/README_ja.md)

A video downloader rewritten in Rust from the Python `downloader.py`. Downloads videos from multiple platforms using yt-dlp.

## Features

- 🚀 **Automatic yt-dlp Download**: Automatically downloads yt-dlp from GitHub Releases to `./binaries/` if not installed on the system
- 🎯 **Platform Auto-Detection**: Automatically detects Twitch, YouTube, Twitter/X from URLs and uses optimal settings
- 🔄 **3 Operating Modes**: Interactive loop mode, single URL mode, and batch mode
- ⚙️ **Detailed Customization**: output directory, quality, format, audio extraction, subtitle options, and more
- 🍪 **Cookie Support**: Browser cookie authentication support
- 📦 **Single Executable**: Runs as a single compiled Rust executable
- ⚡ **Fast & Lightweight**: High performance with Rust

## Supported Platforms

- **YouTube** (youtube.com, youtu.be)
  - Chrome cookie authentication
  - Best quality (bestvideo+bestaudio)
  - Thumbnail & metadata embedding
  - Processed as access from Japan

- **Twitch** (twitch.tv)
  - Saved in 1080p60
  - Thumbnail & metadata embedding

- **Twitter/X** (twitter.com, x.com)
  - Saved in MP4 format
  - Thumbnail & metadata embedding

- **Other Sites**
  - Best quality priority (`bv*+ba/b`)
  - Subtitle download only when explicitly requested via options
  - Chrome cookie authentication (default)

## Installation

### Using Pre-built Binaries

```bash
# Release build
cargo build --release

# Executable is generated at target/release/downloader.exe
```

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd Downloader

# Release build
cargo build --release
```

## Usage

### Mode 1: Interactive Loop Mode (Default)

Launch without arguments to download multiple URLs continuously.

```bash
.\target\release\downloader.exe

# Enter URLs continuously
URL> https://www.youtube.com/watch?v=...
URL> https://www.twitch.tv/videos/...
URL> exit  # Or quit, Ctrl+C to exit
```

**Exit methods:**
- Type `exit` or `quit`
- Force exit with Ctrl+C
- EOF with Ctrl+Z (Windows) or Ctrl+D (Unix)

### Mode 2: Single URL Mode

Download one URL and exit.

```bash
.\target\release\downloader.exe --url "https://www.youtube.com/watch?v=..."
```

### Mode 3: Batch Mode

Download multiple URLs at once.

```bash
.\target\release\downloader.exe --urls "https://youtube.com/..." "https://twitch.tv/..." "https://x.com/..."
```

### Help Display

```bash
.\target\release\downloader.exe --help
```

## Changelog

Release notes are maintained in [Changelog.md](./Changelog.md).

## About yt-dlp

This program searches for yt-dlp in the following priority order:

1. **System PATH**: Uses it if `yt-dlp` command is available
2. **Local Binary**: Uses it if `./binaries/yt-dlp.exe` exists
3. **Automatic Download**: Automatically downloads from GitHub Releases if the above are not found

If yt-dlp is not found on first run, it will be downloaded automatically.

## Output Destination

Downloaded videos are saved to the directory where the program is executed (current directory).

Filename: `{video title}.{extension}`

## Dependencies

- [yt-dlp](https://github.com/yt-dlp/yt-dlp) - Video download tool (automatic download)
- Rust 1.70 or higher

## License

BSD-2-Clause
