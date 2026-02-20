use anyhow::{bail, Context, Result};
use clap::Parser;
use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const REPO_OWNER: &str = "darui3018823";
const REPO_NAME: &str = "Downloader";

fn parse_threads(value: &str) -> std::result::Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| "--threads には1以上の整数を指定してください".to_string())?;

    if parsed == 0 {
        return Err("--threads には1以上の整数を指定してください".to_string());
    }

    Ok(parsed)
}

fn sanitize_file_name(input: &str) -> String {
    let invalid = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut result = input
        .chars()
        .map(|c| {
            if invalid.contains(&c) || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect::<String>();

    result = result.trim().to_string();
    if result.is_empty() {
        "download".to_string()
    } else {
        result
    }
}

fn error_log_dir() -> PathBuf {
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        return PathBuf::from(user_profile)
            .join("downloader")
            .join("errorlog");
    }

    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join("downloader").join("errorlog");
    }

    PathBuf::from("./errorlog")
}

fn new_error_log_path(url: &str) -> Result<PathBuf> {
    let log_dir = error_log_dir();
    fs::create_dir_all(&log_dir).context("errorlogディレクトリの作成に失敗しました")?;

    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let url_tag = sanitize_file_name(url).chars().take(48).collect::<String>();
    let file_name = format!("{}_{}.log", epoch, url_tag);

    Ok(log_dir.join(file_name))
}

fn append_log(log_path: &Path, message: &str) {
    let mut file = match OpenOptions::new().create(true).append(true).open(log_path) {
        Ok(file) => file,
        Err(_) => return,
    };

    let _ = writeln!(file, "{}", message);
}

/// yt-dlpを使用した動画ダウンローダー
#[derive(Parser)]
#[command(name = "downloader")]
#[command(version = "2.0.0-beta.4")]
#[command(about = "yt-dlpを使用した動画ダウンローダー", long_about = None)]
struct Cli {
    /// 単一URLをダウンロードして終了
    #[arg(long)]
    url: Option<String>,

    /// 複数のURLを一度にダウンロード
    #[arg(long, num_args = 1..)]
    urls: Option<Vec<String>>,

    /// ダウンロード先ディレクトリ
    #[arg(short = 'o', long, default_value = "./")]
    output_dir: String,

    /// 音声のみダウンロード（mp3形式）
    #[arg(short = 'a', long)]
    audio_only: bool,

    /// 画質指定 (best, 1080p, 720p, 480p, 360p)
    #[arg(long)]
    quality: Option<String>,

    /// 出力フォーマット (mp4, mkv, webm)
    #[arg(short = 'f', long)]
    format: Option<String>,

    /// サムネイル・メタデータの埋め込みをスキップ
    #[arg(long)]
    no_metadata: bool,

    /// クッキー元のブラウザ (chrome, firefox, edge, safari)
    /// 指定しない場合はクッキーを使用しません
    #[arg(long)]
    cookies: Option<String>,

    /// プレイリスト全体をダウンロード
    #[arg(long)]
    playlist: bool,

    /// 字幕をダウンロード
    #[arg(long)]
    write_sub: bool,

    /// 字幕言語 (例: ja,en,all)
    #[arg(long)]
    sub_lang: Option<String>,

    /// 字幕フォーマット (例: srt,vtt,best)
    #[arg(long)]
    sub_format: Option<String>,

    /// 字幕変換フォーマット (例: srt,vtt)
    #[arg(long)]
    convert_subs: Option<String>,

    /// アプリ本体を最新Releaseバイナリに更新
    #[arg(short = 'u', long)]
    update: bool,

    /// yt-dlpを最新バージョンに更新
    #[arg(long)]
    update_ytdlp: bool,

    /// 詳細ログを出力
    #[arg(short = 'v', long)]
    verbose: bool,

    /// 最小限の出力のみ
    #[arg(short = 'q', long)]
    quiet: bool,

    /// バッチモード時の最大スレッド数（--urls 専用）
    #[arg(short = 't', long, value_parser = parse_threads)]
    threads: Option<usize>,

    /// 抽出のみyt-dlpを使い、ダウンロードはRustで実行（--url 専用・実験的）
    #[arg(long)]
    rust_download: bool,

    /// クレジット情報を表示
    #[arg(long)]
    credit: bool,
}

/// ダウンロード設定
#[derive(Debug, Clone)]
struct DownloadConfig {
    output_dir: String,
    audio_only: bool,
    quality: Option<String>,
    format: String,
    no_metadata: bool,
    cookies: Option<String>,
    playlist: bool,
    write_sub: bool,
    sub_lang: Option<String>,
    sub_format: Option<String>,
    convert_subs: Option<String>,
    verbose: bool,
    quiet: bool,
    threads: Option<usize>,
    rust_download: bool,
}

impl DownloadConfig {
    fn from_cli(cli: &Cli) -> Self {
        Self {
            output_dir: cli.output_dir.clone(),
            audio_only: cli.audio_only,
            quality: cli.quality.clone(),
            format: cli.format.clone().unwrap_or_else(|| "mp4".to_string()),
            no_metadata: cli.no_metadata,
            cookies: cli.cookies.clone(),
            playlist: cli.playlist,
            write_sub: cli.write_sub,
            sub_lang: cli.sub_lang.clone(),
            sub_format: cli.sub_format.clone(),
            convert_subs: cli.convert_subs.clone(),
            verbose: cli.verbose,
            quiet: cli.quiet,
            threads: cli.threads,
            rust_download: cli.rust_download,
        }
    }
}

/// クレジット情報を表示
fn show_credits() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                 Video Downloader v2-beta4                    ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  A Rust-based video downloader powered by yt-dlp             ║");
    println!("║                                                              ║");
    println!("║  Author: darui3018823                                        ║");
    println!("║  GitHub: https://github.com/darui3018823/Downloader          ║");
    println!("║                                                              ║");
    println!("║  Original Python version: downloader.py                      ║");
    println!("║  Rust rewrite: v1.0.0 - Complete rewrite in Rust             ║");
    println!("║                v1.1.0 - CLI enhancements                     ║");
    println!("║                v1.2.0 - Advanced options                     ║");
    println!("║                v1.3.0 - Changelog migration                  ║");
    println!("║                v1.3.2 - Platform custom expansion            ║");
    println!("║                v1.3.3 - Batch threading control              ║");
    println!("║                v2-beta1 - Rust download experiment           ║");
    println!("║                v2-beta2 - Rust extract fallback fix          ║");
    println!("║                v2-beta3 - Split stream merge                 ║");
    println!("║                v2-beta4 - Parallel progress output           ║");
    println!("║                                                              ║");
    println!("║  Powered by:                                                 ║");
    println!("║    • yt-dlp (https://github.com/yt-dlp/yt-dlp)               ║");
    println!("║    • Rust programming language                               ║");
    println!("║    • clap - CLI argument parsing                             ║");
    println!("║    • reqwest - HTTP client                                   ║");
    println!("║    • anyhow - Error handling                                 ║");
    println!("║                                                              ║");
    println!("║  License: BSD-2-Clause                                       ║");
    println!("║                                                              ║");
    println!("║  Features:                                                   ║");
    println!("║    ✓ Auto-download yt-dlp from GitHub Releases               ║");
    println!("║    ✓ Platform detection (YouTube, Twitch, Twitter/X)         ║");
    println!("║    ✓ Interactive loop mode                                   ║");
    println!("║    ✓ Single URL & Batch download modes                       ║");
    println!("║    ✓ Audio-only download (mp3)                               ║");
    println!("║    ✓ Quality & format selection                              ║");
    println!("║    ✓ Playlist support                                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}

fn select_latest_asset(assets: &[Value]) -> Option<(String, String)> {
    let matches_current_platform = |name: &str| -> bool {
        let lower = name.to_ascii_lowercase();
        if cfg!(windows) {
            lower.ends_with(".exe")
        } else if cfg!(target_os = "macos") {
            lower.contains("mac") || lower.contains("darwin")
        } else {
            lower.contains("linux") || !lower.ends_with(".exe")
        }
    };

    let mut fallback: Option<(String, String)> = None;

    for asset in assets {
        let name = asset.get("name")?.as_str()?;
        let url = asset.get("browser_download_url")?.as_str()?;
        let lower = name.to_ascii_lowercase();

        if !lower.contains("downloader") {
            continue;
        }

        let candidate = (name.to_string(), url.to_string());
        if matches_current_platform(name) {
            return Some(candidate);
        }

        if fallback.is_none() {
            fallback = Some(candidate);
        }
    }

    fallback
}

fn update_release_binary() -> Result<()> {
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        REPO_OWNER, REPO_NAME
    );

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .context("HTTPクライアントの初期化に失敗しました")?;

    let response = client
        .get(&api_url)
        .header("User-Agent", "downloader-self-updater")
        .send()
        .context("最新Release情報の取得に失敗しました")?;

    if !response.status().is_success() {
        bail!(
            "最新Release情報の取得に失敗しました (status: {})",
            response.status()
        );
    }

    let release: Value = response
        .json()
        .context("最新Release情報の解析に失敗しました")?;

    let tag_name = release
        .get("tag_name")
        .and_then(Value::as_str)
        .unwrap_or("unknown");

    let current_tag = format!("v{}", env!("CARGO_PKG_VERSION"));
    if tag_name == current_tag {
        println!("すでに最新バージョンです: {}", current_tag);
        return Ok(());
    }

    let assets = release
        .get("assets")
        .and_then(Value::as_array)
        .context("Releaseにアセットがありません")?;

    let (asset_name, download_url) =
        select_latest_asset(assets).context("現在の環境向けバイナリが見つかりません")?;

    println!("最新Release: {}", tag_name);
    println!("ダウンロード対象: {}", asset_name);

    let binary_response = client
        .get(&download_url)
        .header("User-Agent", "downloader-self-updater")
        .send()
        .context("Releaseバイナリのダウンロードに失敗しました")?;

    if !binary_response.status().is_success() {
        bail!(
            "Releaseバイナリのダウンロードに失敗しました (status: {})",
            binary_response.status()
        );
    }

    let binary_data = binary_response
        .bytes()
        .context("バイナリデータの読み込みに失敗しました")?;

    let current_exe = std::env::current_exe().context("実行ファイルパスの取得に失敗しました")?;
    let current_dir = current_exe
        .parent()
        .context("実行ファイルのディレクトリ取得に失敗しました")?;

    let asset_file_name = Path::new(&asset_name)
        .file_name()
        .and_then(|name| name.to_str())
        .context("Releaseアセット名の解析に失敗しました")?;

    let downloaded_asset_path = current_dir.join(asset_file_name);
    let staged_path = if downloaded_asset_path == current_exe {
        if cfg!(windows) {
            current_exe.with_extension("new.exe")
        } else {
            current_exe.with_extension("new")
        }
    } else {
        downloaded_asset_path.clone()
    };

    if staged_path.exists() {
        fs::remove_file(&staged_path).context("既存の更新バイナリ削除に失敗しました")?;
    }

    fs::write(&staged_path, &binary_data).context("新しいバイナリの保存に失敗しました")?;

    println!(
        "リネーム更新: {} -> {}",
        staged_path.display(),
        current_exe.display()
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&staged_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&staged_path, perms)?;

        fs::rename(&staged_path, &current_exe).context("バイナリの差し替えに失敗しました")?;
        println!("✓ 更新が完了しました。再実行してください。");
    }

    #[cfg(windows)]
    {
        let update_script = current_exe.with_extension("update.cmd");
        let script = format!(
            "@echo off\r\nping 127.0.0.1 -n 3 >nul\r\nmove /Y \"{}\" \"{}\" >nul\r\ndel \"%~f0\"\r\n",
            staged_path.display(),
            current_exe.display()
        );

        fs::write(&update_script, script).context("更新スクリプトの作成に失敗しました")?;

        Command::new("cmd")
            .arg("/C")
            .arg(format!("start \"\" /B \"{}\"", update_script.display()))
            .spawn()
            .context("更新スクリプトの起動に失敗しました")?;

        println!("✓ 更新処理を開始しました。終了後にバイナリが差し替わります。");
    }

    Ok(())
}

/// yt-dlpバイナリのパスを取得または自動ダウンロード
fn ensure_ytdlp(force_update: bool) -> Result<PathBuf> {
    let binaries_dir = PathBuf::from("./binaries");
    let ytdlp_path = binaries_dir.join(if cfg!(windows) {
        "yt-dlp.exe"
    } else {
        "yt-dlp"
    });

    // 強制更新の場合は既存ファイルを削除
    if force_update && ytdlp_path.exists() {
        println!("既存のyt-dlpを削除しています...");
        fs::remove_file(&ytdlp_path).context("既存ファイルの削除に失敗しました")?;
    }

    // まず環境のPATHからyt-dlpを探す（更新時を除く）
    if !force_update {
        if let Ok(output) = Command::new("yt-dlp").arg("--version").output() {
            if output.status.success() {
                println!("✓ 環境からyt-dlpを検出しました");
                return Ok(PathBuf::from("yt-dlp"));
            }
        }
    }

    // ローカルバイナリを確認
    if ytdlp_path.exists() && !force_update {
        println!("✓ {}からyt-dlpを検出しました", ytdlp_path.display());
        return Ok(ytdlp_path);
    }

    // GitHubからダウンロード
    if force_update {
        println!("yt-dlpを最新バージョンに更新しています...");
    } else {
        println!("yt-dlpが見つかりません。GitHubからダウンロードしています...");
    }
    download_ytdlp(&binaries_dir, &ytdlp_path)?;

    Ok(ytdlp_path)
}

/// GitHubのReleasesからyt-dlpをダウンロード
fn download_ytdlp(binaries_dir: &Path, ytdlp_path: &Path) -> Result<()> {
    // binariesディレクトリを作成
    fs::create_dir_all(binaries_dir).context("binariesディレクトリの作成に失敗しました")?;

    // プラットフォームに応じたダウンロードURL
    let download_url = if cfg!(windows) {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
    } else if cfg!(target_os = "macos") {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
    } else {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp"
    };

    println!("ダウンロード中: {}", download_url);

    // ファイルをダウンロード
    let response =
        reqwest::blocking::get(download_url).context("yt-dlpのダウンロードに失敗しました")?;

    if !response.status().is_success() {
        bail!("ダウンロードエラー: ステータスコード {}", response.status());
    }

    let bytes = response
        .bytes()
        .context("レスポンスの読み取りに失敗しました")?;

    // ファイルに書き込み
    fs::write(ytdlp_path, &bytes).context("yt-dlpの保存に失敗しました")?;

    // Unix系OSの場合、実行権限を付与
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(ytdlp_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(ytdlp_path, perms)?;
    }

    println!("✓ yt-dlpを{}に保存しました", ytdlp_path.display());
    Ok(())
}

/// プラットフォームを検出
#[derive(Debug)]
enum Platform {
    Twitch,
    YouTube,
    Twitter,
    Niconico,
    SoundCloud,
    Instagram,
    TikTok,
    Bilibili,
    Generic,
}

impl Platform {
    fn detect(url: &str) -> Self {
        let lower = url.to_ascii_lowercase();

        if lower.contains("twitch.tv") {
            Platform::Twitch
        } else if lower.contains("youtube.com") || lower.contains("youtu.be") {
            Platform::YouTube
        } else if lower.contains("twitter.com") || lower.contains("x.com") {
            Platform::Twitter
        } else if lower.contains("nicovideo.jp") || lower.contains("nico.ms") {
            Platform::Niconico
        } else if lower.contains("soundcloud.com") {
            Platform::SoundCloud
        } else if lower.contains("instagram.com") {
            Platform::Instagram
        } else if lower.contains("tiktok.com") {
            Platform::TikTok
        } else if lower.contains("bilibili.com") || lower.contains("b23.tv") {
            Platform::Bilibili
        } else {
            Platform::Generic
        }
    }
}

/// プラットフォームに応じたyt-dlpコマンドを構築
fn build_command(
    ytdlp_path: &Path,
    platform: Platform,
    url: &str,
    config: &DownloadConfig,
) -> Command {
    let mut cmd = Command::new(ytdlp_path);

    // 出力先ディレクトリを作成
    if let Err(e) = fs::create_dir_all(&config.output_dir) {
        eprintln!("警告: 出力ディレクトリの作成に失敗: {}", e);
    }

    let output_template = format!("{}/%(title)s.%(ext)s", config.output_dir);

    // 音声のみモード
    if config.audio_only {
        cmd.args(["-x", "--audio-format", "mp3"]);
        cmd.args(["--output", &output_template, url]);

        // 詳細ログ / 静寂モード
        if config.verbose {
            cmd.arg("--verbose");
        } else if config.quiet {
            cmd.arg("--quiet");
        }

        return cmd;
    }

    // 画質指定
    let format_arg = if let Some(quality) = &config.quality {
        match quality.as_str() {
            "best" => "bestvideo+bestaudio",
            q => q, // 1080p, 720p, etc.
        }
    } else {
        // プラットフォーム別のデフォルト画質
        match platform {
            Platform::Twitch => "1080p60+bestaudio",
            Platform::YouTube => "bestvideo+bestaudio",
            Platform::Twitter => "bestvideo+bestaudio/best",
            Platform::Niconico => "bestvideo+bestaudio/best",
            Platform::SoundCloud => "bestaudio/best",
            Platform::Instagram => "bestvideo+bestaudio/best",
            Platform::TikTok => "bestvideo+bestaudio/best",
            Platform::Bilibili => "bv*+ba/b",
            Platform::Generic => "bv*+ba/b",
        }
    };

    cmd.args(["-f", format_arg]);
    if !matches!(platform, Platform::SoundCloud) {
        cmd.args(["--merge-output-format", &config.format]);
    }

    // メタデータ
    if !config.no_metadata {
        cmd.args(["--embed-thumbnail", "--add-metadata"]);
    }

    // クッキー（指定された場合のみ）
    if let Some(ref cookies) = config.cookies {
        cmd.args(["--cookies-from-browser", cookies]);
    }

    // プレイリスト
    if !config.playlist {
        cmd.arg("--no-playlist");
    }

    // プラットフォーム固有の設定
    match platform {
        Platform::YouTube => {
            cmd.args(["-4", "--geo-bypass-country", "JP"]);
        }
        Platform::Niconico => {
            cmd.args(["--geo-bypass-country", "JP", "--ignore-errors"]);
        }
        Platform::Instagram | Platform::TikTok => {
            cmd.arg("--user-agent");
            cmd.arg("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.74 Safari/537.36");
            cmd.arg("--ignore-errors");
        }
        Platform::Bilibili => {
            cmd.arg("--ignore-errors");
        }
        Platform::Generic => {
            cmd.args(["--geo-bypass-country", "JP"]);
            cmd.arg("--user-agent");
            cmd.arg("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.74 Safari/537.36");

            if config.write_sub {
                cmd.arg("--write-sub");

                if let Some(sub_lang) = &config.sub_lang {
                    cmd.args(["--sub-lang", sub_lang]);
                }

                if let Some(sub_format) = &config.sub_format {
                    cmd.args(["--sub-format", sub_format]);
                }

                if let Some(convert_subs) = &config.convert_subs {
                    cmd.args(["--convert-subs", convert_subs]);
                }
            }

            cmd.arg("--ignore-errors");
        }
        _ => {}
    }

    // 詳細ログ / 静寂モード
    if config.verbose {
        cmd.arg("--verbose");
    } else if config.quiet {
        cmd.arg("--quiet");
    }

    cmd.args(["--output", &output_template, url]);
    cmd
}

fn execute_download_command(mut cmd: Command, suppress_ytdlp_output: bool) -> Result<()> {
    if suppress_ytdlp_output {
        let output = cmd.output().context("yt-dlpの実行に失敗しました")?;

        if output.status.success() {
            Ok(())
        } else {
            bail!(
                "yt-dlpがエラーコード{}で終了しました",
                output.status.code().unwrap_or(-1)
            );
        }
    } else {
        let status = cmd.status().context("yt-dlpの実行に失敗しました")?;

        if status.success() {
            Ok(())
        } else {
            bail!(
                "yt-dlpがエラーコード{}で終了しました",
                status.code().unwrap_or(-1)
            );
        }
    }
}

#[derive(Debug, Clone)]
struct RustMediaStream {
    media_url: String,
    ext: String,
    protocol: String,
    headers: Vec<(String, String)>,
    format_id: Option<String>,
    vcodec: Option<String>,
    acodec: Option<String>,
    has_video: bool,
    has_audio: bool,
    has_fragments: bool,
    score: f64,
}

#[derive(Debug)]
struct RustDownloadCandidate {
    title: String,
    output_ext: String,
    single_stream: Option<RustMediaStream>,
    video_stream: Option<RustMediaStream>,
    audio_stream: Option<RustMediaStream>,
}

fn is_stream_protocol(protocol: &str) -> bool {
    let lower = protocol.to_ascii_lowercase();
    lower.contains("m3u8")
        || lower.contains("dash")
        || lower.contains("hls")
        || lower.contains("fragment")
}

fn stream_from_value(value: &Value) -> Option<RustMediaStream> {
    let media_url = value.get("url")?.as_str()?.to_string();
    if media_url.is_empty() {
        return None;
    }

    let protocol = value
        .get("protocol")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let has_fragments = value
        .get("fragments")
        .and_then(Value::as_array)
        .map(|arr| !arr.is_empty())
        .unwrap_or(false);

    let vcodec = value
        .get("vcodec")
        .and_then(Value::as_str)
        .map(|s| s.to_string());
    let acodec = value
        .get("acodec")
        .and_then(Value::as_str)
        .map(|s| s.to_string());

    let has_video = vcodec.as_deref().unwrap_or("none") != "none";
    let has_audio = acodec.as_deref().unwrap_or("none") != "none";

    let ext = value
        .get("ext")
        .and_then(Value::as_str)
        .unwrap_or("bin")
        .to_string();
    let format_id = value
        .get("format_id")
        .and_then(Value::as_str)
        .map(|s| s.to_string());

    let mut headers = Vec::new();
    if let Some(map) = value.get("http_headers").and_then(Value::as_object) {
        for (key, val) in map {
            if let Some(text) = val.as_str() {
                headers.push((key.clone(), text.to_string()));
            }
        }
    }

    let score = value
        .get("tbr")
        .and_then(Value::as_f64)
        .or_else(|| value.get("abr").and_then(Value::as_f64))
        .or_else(|| value.get("vbr").and_then(Value::as_f64))
        .unwrap_or(0.0);

    Some(RustMediaStream {
        media_url,
        ext,
        protocol,
        headers,
        format_id,
        vcodec,
        acodec,
        has_video,
        has_audio,
        has_fragments,
        score,
    })
}

fn is_usable_stream(stream: &RustMediaStream) -> bool {
    !stream.media_url.is_empty() && !stream.has_fragments && !is_stream_protocol(&stream.protocol)
}

fn best_stream<'a, F>(streams: &'a [RustMediaStream], predicate: F) -> Option<&'a RustMediaStream>
where
    F: Fn(&RustMediaStream) -> bool,
{
    streams
        .iter()
        .filter(|s| predicate(s))
        .max_by(|a, b| a.score.total_cmp(&b.score))
}

fn extract_candidate_from_json(
    metadata: &Value,
    config: &DownloadConfig,
) -> Result<RustDownloadCandidate> {
    let title = metadata
        .get("title")
        .and_then(Value::as_str)
        .map(sanitize_file_name)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            metadata
                .get("id")
                .and_then(Value::as_str)
                .map(sanitize_file_name)
                .unwrap_or_else(|| "download".to_string())
        });

    let mut streams = Vec::new();

    if let Some(requested_formats) = metadata.get("requested_formats").and_then(Value::as_array) {
        for value in requested_formats {
            if let Some(stream) = stream_from_value(value) {
                streams.push(stream);
            }
        }
    }

    if streams.is_empty() {
        if let Some(formats) = metadata.get("formats").and_then(Value::as_array) {
            for value in formats {
                if let Some(stream) = stream_from_value(value) {
                    streams.push(stream);
                }
            }
        }
    }

    if streams.is_empty() {
        if let Some(single) = stream_from_value(metadata) {
            streams.push(single);
        }
    }

    if streams.is_empty() {
        bail!("抽出JSONに直リンクURLがありません");
    }

    if config.audio_only {
        let selected = best_stream(&streams, |s| {
            is_usable_stream(s) && s.has_audio && !s.has_video
        })
        .or_else(|| best_stream(&streams, |s| is_usable_stream(s) && s.has_audio))
        .cloned()
        .context("Rust audio-onlyモードで利用可能な音声ストリームが見つかりません")?;

        return Ok(RustDownloadCandidate {
            title,
            output_ext: selected.ext.clone(),
            single_stream: Some(selected),
            video_stream: None,
            audio_stream: None,
        });
    }

    let video_stream = best_stream(&streams, |s| {
        is_usable_stream(s) && s.has_video && !s.has_audio
    })
    .cloned();
    let audio_stream = best_stream(&streams, |s| {
        is_usable_stream(s) && s.has_audio && !s.has_video
    })
    .cloned();

    if let (Some(video), Some(audio)) = (video_stream, audio_stream) {
        return Ok(RustDownloadCandidate {
            title,
            output_ext: config.format.clone(),
            single_stream: None,
            video_stream: Some(video),
            audio_stream: Some(audio),
        });
    }

    let single_stream = best_stream(&streams, |s| {
        is_usable_stream(s) && s.has_video && s.has_audio
    })
    .or_else(|| best_stream(&streams, is_usable_stream))
    .cloned()
    .context("Rustモードで利用可能な単一ストリームが見つかりません")?;

    Ok(RustDownloadCandidate {
        title,
        output_ext: single_stream.ext.clone(),
        single_stream: Some(single_stream),
        video_stream: None,
        audio_stream: None,
    })
}

fn extract_with_ytdlp(
    ytdlp_path: &Path,
    url: &str,
    config: &DownloadConfig,
    log_path: &Path,
) -> Result<Value> {
    let mut cmd = Command::new(ytdlp_path);
    cmd.args(["-J", "--no-playlist"]);

    if config.audio_only {
        cmd.args(["-f", "bestaudio/best"]);
    } else {
        cmd.args(["-f", "best/bv*+ba/b"]);
    }

    if let Some(cookies) = &config.cookies {
        cmd.args(["--cookies-from-browser", cookies]);
    }

    cmd.arg(url);

    append_log(log_path, &format!("[extract] command: {:?}", cmd));
    let output = cmd.output().context("yt-dlp抽出の実行に失敗しました")?;

    append_log(
        log_path,
        &format!("[extract] exit: {:?}", output.status.code()),
    );
    append_log(
        log_path,
        &format!(
            "[extract] stderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ),
    );

    if !output.status.success() {
        bail!(
            "yt-dlp抽出が失敗しました。詳細はログを確認してください: {}",
            log_path.display()
        );
    }

    let stdout = String::from_utf8(output.stdout).context("抽出JSONのUTF-8変換に失敗しました")?;
    append_log(log_path, &format!("[extract] stdout_len: {}", stdout.len()));

    let metadata: Value =
        serde_json::from_str(&stdout).context("yt-dlp抽出JSONの解析に失敗しました")?;
    append_log(
        log_path,
        &format!(
            "[extract] json:\n{}",
            serde_json::to_string_pretty(&metadata)
                .unwrap_or_else(|_| "<json pretty print failed>".to_string())
        ),
    );

    Ok(metadata)
}

fn rust_download_stream(
    stream: &RustMediaStream,
    output_path: &Path,
    log_path: &Path,
    progress_label: &str,
    show_progress: bool,
) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .build()
        .context("HTTPクライアントの作成に失敗しました")?;

    let mut request = client.get(&stream.media_url);
    for (key, value) in &stream.headers {
        request = request.header(key, value);
    }

    append_log(
        log_path,
        &format!("[download] format_id: {:?}", stream.format_id),
    );
    append_log(log_path, &format!("[download] vcodec: {:?}", stream.vcodec));
    append_log(log_path, &format!("[download] acodec: {:?}", stream.acodec));
    append_log(log_path, &format!("[download] url: {}", stream.media_url));
    append_log(
        log_path,
        &format!("[download] protocol: {}", stream.protocol),
    );
    append_log(
        log_path,
        &format!("[download] output: {}", output_path.display()),
    );
    append_log(
        log_path,
        &format!("[download] headers_from_extract: {:?}", stream.headers),
    );

    let mut response = request
        .send()
        .context("Rustダウンロード要求に失敗しました")?;
    append_log(
        log_path,
        &format!("[download] status: {}", response.status()),
    );
    append_log(
        log_path,
        &format!("[download] response_headers: {:?}", response.headers()),
    );

    if !response.status().is_success() {
        bail!("HTTPステータスが異常です: {}", response.status());
    }

    let total_size = response.content_length();
    if show_progress {
        if let Some(total) = total_size {
            println!("[{}] 開始: {} bytes", progress_label, total);
        } else {
            println!("[{}] 開始", progress_label);
        }
    }

    let mut file = fs::File::create(output_path)
        .with_context(|| format!("出力ファイル作成に失敗しました: {}", output_path.display()))?;
    let mut buf = [0u8; 64 * 1024];
    let mut total_bytes: u64 = 0;
    let mut last_reported_mb: u64 = 0;
    let mut last_percent_reported: u64 = 0;

    loop {
        let read = response
            .read(&mut buf)
            .context("レスポンス読み取りに失敗しました")?;
        if read == 0 {
            break;
        }

        file.write_all(&buf[..read])
            .context("出力ファイル書き込みに失敗しました")?;
        total_bytes += read as u64;

        let current_mb = total_bytes / (1024 * 1024);
        if current_mb >= last_reported_mb + 10 {
            last_reported_mb = current_mb;
            append_log(
                log_path,
                &format!("[download] progress_bytes: {}", total_bytes),
            );
        }

        if show_progress {
            if let Some(total) = total_size {
                if total > 0 {
                    let percent = (total_bytes.saturating_mul(100) / total).min(100);
                    if percent >= last_percent_reported + 5 || percent == 100 {
                        last_percent_reported = percent;
                        println!("[{}] {}%", progress_label, percent);
                    }
                }
            }
        }
    }

    append_log(
        log_path,
        &format!("[download] completed_bytes: {}", total_bytes),
    );

    if show_progress {
        println!("[{}] 完了", progress_label);
    }

    Ok(())
}

fn ffmpeg_merge_streams(
    video_path: &Path,
    audio_path: &Path,
    output_path: &Path,
    log_path: &Path,
) -> Result<()> {
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-hide_banner",
        "-y",
        "-i",
        &video_path.to_string_lossy(),
        "-i",
        &audio_path.to_string_lossy(),
        "-c",
        "copy",
        &output_path.to_string_lossy(),
    ]);

    append_log(log_path, &format!("[ffmpeg] command: {:?}", cmd));
    let output = cmd.output().context("ffmpeg結合の実行に失敗しました")?;
    append_log(
        log_path,
        &format!("[ffmpeg] exit: {:?}", output.status.code()),
    );
    append_log(
        log_path,
        &format!(
            "[ffmpeg] stderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ),
    );

    if !output.status.success() {
        bail!(
            "ffmpeg結合に失敗しました (code: {:?})",
            output.status.code()
        );
    }

    Ok(())
}

fn download_single_rust(ytdlp_path: &Path, url: &str, config: &DownloadConfig) -> Result<()> {
    let log_path = new_error_log_path(url)?;
    append_log(&log_path, "=== rust-download mode start ===");
    append_log(&log_path, &format!("[input] url: {}", url));
    append_log(&log_path, &format!("[config] {:?}", config));

    if !config.quiet {
        println!("[flow] 1/4 抽出中...");
    }

    let metadata = extract_with_ytdlp(ytdlp_path, url, config, &log_path)
        .with_context(|| format!("抽出処理に失敗しました。ログ: {}", log_path.display()))?;
    let candidate = extract_candidate_from_json(&metadata, config)
        .with_context(|| format!("抽出結果の解釈に失敗しました。ログ: {}", log_path.display()))?;

    fs::create_dir_all(&config.output_dir).context("出力ディレクトリの作成に失敗しました")?;
    let file_name = format!("{}.{}", candidate.title, candidate.output_ext);
    let output_path = PathBuf::from(&config.output_dir).join(file_name);

    if !config.quiet {
        println!("Rust download mode: {}", output_path.display());
        println!("詳細ログ: {}", log_path.display());
    }

    if let Some(single_stream) = &candidate.single_stream {
        if !config.quiet {
            println!("[flow] 2/4 単一ストリームDL中...");
        }

        rust_download_stream(
            single_stream,
            &output_path,
            &log_path,
            "single",
            !config.quiet,
        )
        .with_context(|| {
            format!(
                "Rustダウンロード失敗。ハング/失敗時は --rust-download を外してください。ログ: {}",
                log_path.display()
            )
        })?;
    } else if let (Some(video_stream), Some(audio_stream)) =
        (&candidate.video_stream, &candidate.audio_stream)
    {
        let temp_video = output_path.with_extension("video.tmp");
        let temp_audio = output_path.with_extension("audio.tmp");

        if !config.quiet {
            println!("[flow] 2/4 動画/音声の並列DL中...");
        }

        let video_stream = video_stream.clone();
        let audio_stream = audio_stream.clone();
        let video_log = log_path.clone();
        let audio_log = log_path.clone();
        let video_path = temp_video.clone();
        let audio_path = temp_audio.clone();
        let show_progress = !config.quiet;

        let video_handle = thread::spawn(move || {
            rust_download_stream(
                &video_stream,
                &video_path,
                &video_log,
                "video",
                show_progress,
            )
        });

        let audio_handle = thread::spawn(move || {
            rust_download_stream(
                &audio_stream,
                &audio_path,
                &audio_log,
                "audio",
                show_progress,
            )
        });

        let video_result = video_handle
            .join()
            .map_err(|_| anyhow::anyhow!("動画DLスレッドがpanicしました"))?;
        video_result.with_context(|| {
            format!(
                "動画ストリームDL失敗。ハング/失敗時は --rust-download を外してください。ログ: {}",
                log_path.display()
            )
        })?;

        let audio_result = audio_handle
            .join()
            .map_err(|_| anyhow::anyhow!("音声DLスレッドがpanicしました"))?;
        audio_result.with_context(|| {
            format!(
                "音声ストリームDL失敗。ハング/失敗時は --rust-download を外してください。ログ: {}",
                log_path.display()
            )
        })?;

        if !config.quiet {
            println!("[flow] 3/4 ffmpeg結合中...");
        }

        ffmpeg_merge_streams(&temp_video, &temp_audio, &output_path, &log_path).with_context(
            || {
                format!(
                    "ffmpeg結合失敗。ハング/失敗時は --rust-download を外してください。ログ: {}",
                    log_path.display()
                )
            },
        )?;

        let _ = fs::remove_file(&temp_video);
        let _ = fs::remove_file(&temp_audio);
    } else {
        bail!(
            "Rustダウンロード候補の解釈に失敗しました。ハング/失敗時は --rust-download を外してください"
        );
    }

    append_log(&log_path, "=== rust-download mode done ===");

    if !config.quiet {
        println!("[flow] 4/4 完了");
        println!("\n✓ Rustダウンロードが完了しました。\n");
    }

    Ok(())
}

/// URLをダウンロード
fn download_url(ytdlp_path: &Path, url: &str, config: &DownloadConfig) -> Result<()> {
    if url.trim().is_empty() {
        return Ok(());
    }

    // プラットフォームを検出
    let platform = Platform::detect(url);

    if !config.quiet {
        println!("検出されたプラットフォーム: {:?}", platform);
    }

    // コマンドを構築して実行
    let cmd = build_command(ytdlp_path, platform, url, config);

    if !config.quiet {
        println!("ダウンロードを開始します...\n");
    }

    execute_download_command(cmd, false)?;

    if !config.quiet {
        println!("\n✓ ダウンロードが完了しました。\n");
    }

    Ok(())
}

/// 単一URLモード
fn download_single(ytdlp_path: &Path, url: &str, config: &DownloadConfig) -> Result<()> {
    if !config.quiet {
        println!("=== 単一URLモード ===\n");
    }

    if config.rust_download {
        return download_single_rust(ytdlp_path, url, config);
    }

    download_url(ytdlp_path, url, config)
}

/// バッチモード
fn download_batch(ytdlp_path: &Path, urls: &[String], config: &DownloadConfig) -> Result<()> {
    if !config.quiet {
        println!("=== バッチモード ({} URLs) ===\n", urls.len());
        println!("yt-dlpログは非表示で、スレッド並列実行します。\n");
    }

    let default_workers = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .max(1);
    let configured_workers = config.threads.unwrap_or(default_workers).max(1);
    let max_workers = configured_workers.min(urls.len().max(1));

    if !config.quiet {
        println!(
            "スレッド数: {} (指定: {}, URL数: {})\n",
            max_workers,
            configured_workers,
            urls.len()
        );
    }

    let mut completed = 0usize;
    let mut failed = 0usize;

    for chunk in urls.chunks(max_workers) {
        let mut handles = Vec::with_capacity(chunk.len());

        for url in chunk {
            let ytdlp_path = ytdlp_path.to_path_buf();
            let config = config.clone();
            let url = url.clone();

            handles.push(thread::spawn(move || {
                if url.trim().is_empty() {
                    return (url, Ok(()));
                }

                let platform = Platform::detect(&url);
                let cmd = build_command(&ytdlp_path, platform, &url, &config);
                let result = execute_download_command(cmd, true);
                (url, result)
            }));
        }

        for handle in handles {
            match handle.join() {
                Ok((url, Ok(()))) => {
                    completed += 1;
                    if !config.quiet {
                        println!("[{}/{}] 完了: {}", completed + failed, urls.len(), url);
                    }
                }
                Ok((url, Err(e))) => {
                    failed += 1;
                    eprintln!("エラー ({}): {}", url, e);
                }
                Err(_) => {
                    failed += 1;
                    eprintln!("エラー: ダウンロードスレッドがpanicしました");
                }
            }
        }
    }

    if !config.quiet {
        println!(
            "すべてのダウンロードが完了しました。(成功: {}, 失敗: {})",
            completed, failed
        );
    }
    Ok(())
}

/// 対話的ループモード
fn interactive_loop(ytdlp_path: &Path, config: &DownloadConfig) -> Result<()> {
    if !config.quiet {
        println!("=== 対話的モード ===");
        println!("URLを入力してください (exit/quit で終了, Ctrl+C でも終了可能)\n");
    }

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    loop {
        if !config.quiet {
            print!("URL> ");
            io::stdout().flush()?;
        }

        match lines.next() {
            Some(Ok(input)) => {
                let input = input.trim();

                // 終了コマンドチェック
                if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                    if !config.quiet {
                        println!("終了します。");
                    }
                    break;
                }

                // 空行はスキップ
                if input.is_empty() {
                    continue;
                }

                // URLをダウンロード
                if let Err(e) = download_url(ytdlp_path, input, config) {
                    eprintln!("エラー: {}", e);
                    if !config.quiet {
                        println!("次のURLを入力してください。\n");
                    }
                }
            }
            Some(Err(e)) => {
                eprintln!("入力エラー: {}", e);
                break;
            }
            None => {
                // EOF (Ctrl+D on Unix, Ctrl+Z on Windows) または Ctrl+C
                if !config.quiet {
                    println!("\n終了します。");
                }
                break;
            }
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.update && cli.update_ytdlp {
        bail!("--update と --update-ytdlp は同時に指定できません");
    }

    if cli.rust_download {
        if cli.urls.is_some() || cli.url.is_none() {
            bail!(
                "--rust-download は --url の単一モード専用です（切り分け目的のためフォールバックなし）。ハング/失敗時は --rust-download を外してください"
            );
        }
    }

    // クレジット表示モード
    if cli.credit {
        show_credits();
        return Ok(());
    }

    // 自己更新モード
    if cli.update {
        println!("最新Releaseバイナリへ更新しています...");
        update_release_binary()?;
        return Ok(());
    }

    // yt-dlp更新モード
    if cli.update_ytdlp {
        ensure_ytdlp(true)?;
        println!("\nyt-dlpの更新が完了しました。");
        return Ok(());
    }

    if !cli.quiet {
        println!("=== yt-dlp Video Downloader v2-beta4 ===\n");
    }

    // yt-dlpの確保
    let ytdlp_path = ensure_ytdlp(false)?;

    if !cli.quiet {
        println!();
    }

    // ダウンロード設定を作成
    let config = DownloadConfig::from_cli(&cli);

    // モード判定と実行
    match (cli.url, cli.urls) {
        (Some(url), None) => {
            // 単一URLモード
            download_single(&ytdlp_path, &url, &config)?;
        }
        (None, Some(urls)) => {
            // バッチモード
            download_batch(&ytdlp_path, &urls, &config)?;
        }
        (None, None) => {
            // 対話的ループモード
            interactive_loop(&ytdlp_path, &config)?;
        }
        (Some(_), Some(_)) => {
            bail!("--url と --urls を同時に指定できません");
        }
    }

    Ok(())
}
