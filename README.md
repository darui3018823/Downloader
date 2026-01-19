# Video Downloader (Rust Version)

> 📖 **Languages:** [English](./README.md) | [日本語](./japanese/README_ja.md)

A video downloader rewritten in Rust from the Python `downloader.py`. Downloads videos from multiple platforms using yt-dlp.

## Features

- 🚀 **Automatic yt-dlp Download**: Automatically downloads yt-dlp from GitHub Releases to `./binaries/` if not installed on the system
- 🎯 **Platform Auto-Detection**: Automatically detects Twitch, YouTube, Twitter/X from URLs and uses optimal settings
- 🔄 **3 Operating Modes**: Interactive loop mode, single URL mode, and batch mode
- ⚙️ **Detailed Customization**: 11 options including output directory, quality, format, and audio extraction
- 🍪 **Chrome Cookie Support**: Uses Chrome cookies by default (v1.2.0+)
- 📦 **Single Executable**: Runs as a single compiled Rust executable
- ⚡ **Fast & Lightweight**: High performance with Rust

## Supported Platforms

- **YouTube** (youtube.com, youtu.be)
  - Chrome cookie authentication (default, v1.2.0+)
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
  - Supported with generic settings
  - Subtitle download (SRT format)
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

## v1.2.0 New Features

### Advanced Options

#### Output Directory Specification (`-o` / `--output-dir`)

```bash
# Save to downloads folder
.\target\release\downloader.exe --url "..." -o "./downloads"

# Save to dedicated music folder
.\target\release\downloader.exe -o "./music" -a --url "..."
```

#### Audio Only Download (`-a` / `--audio-only`)

Download audio only in mp3 format.

```bash
.\target\release\downloader.exe --url "..." --audio-only
# Or short form
.\target\release\downloader.exe --url "..." -a
```

#### Quality Specification (`--quality`)

Download with specified resolution.

```bash
# Download in 720p
.\target\release\downloader.exe --url "..." --quality 720p

# Available qualities: best (default), 1080p, 720p, 480p, 360p
```

#### Output Format Specification (`-f` / `--format`)

```bash
# Save in MKV format
.\target\release\downloader.exe --url "..." -f mkv

# Available formats: mp4 (default), mkv, webm
```

#### Skip Metadata (`--no-metadata`)

Skip thumbnail and metadata embedding for faster processing.

```bash
.\target\release\downloader.exe --url "..." --no-metadata
```

#### Cookie Browser Specification (`--cookies`)

**Default changed to Chrome in v1.2.0.**

```bash
# Use Firefox cookies
.\target\release\downloader.exe --url "..." --cookies firefox

# Supported browsers: chrome (default), firefox, edge, safari
```

#### Playlist Download (`--playlist`)

Download entire playlist (default is single video).

```bash
.\target\release\downloader.exe --url "..." --playlist
```

#### yt-dlp Update (`--update-ytdlp`)

Update yt-dlp to the latest version.

```bash
.\target\release\downloader.exe --update-ytdlp
```

#### Verbose Logging (`-v` / `--verbose`)

Output detailed logs for debugging.

```bash
.\target\release\downloader.exe --url "..." -v
```

#### Quiet Mode (`-q` / `--quiet`)

Display minimal output only.

```bash
.\target\release\downloader.exe --url "..." -q
```

#### Credits Display (`--credit`)

Display developer information and credits.

```bash
.\target\release\downloader.exe --credit
```

### Combined Usage Examples

```bash
# Audio only, Firefox cookies, specify output directory
.\target\release\downloader.exe --url "..." -a --cookies firefox -o "./music"

# 720p, quiet mode, no metadata
.\target\release\downloader.exe --url "..." --quality 720p -q --no-metadata

# Playlist, verbose logging, MKV format
.\target\release\downloader.exe --url "..." --playlist -v -f mkv
```

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
