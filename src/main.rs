use anyhow::{bail, Context, Result};
use clap::Parser;
use serde_json::Value;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const REPO_OWNER: &str = "darui3018823";
const REPO_NAME: &str = "Downloader";

/// yt-dlpを使用した動画ダウンローダー
#[derive(Parser)]
#[command(name = "downloader")]
#[command(version = "1.3.2")]
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
        }
    }
}

/// クレジット情報を表示
fn show_credits() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                 Video Downloader v1.3.2                      ║");
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
    let mut cmd = build_command(ytdlp_path, platform, url, config);

    if !config.quiet {
        println!("ダウンロードを開始します...\n");
    }

    let status = cmd.status().context("yt-dlpの実行に失敗しました")?;

    if status.success() {
        if !config.quiet {
            println!("\n✓ ダウンロードが完了しました。\n");
        }
    } else {
        bail!(
            "yt-dlpがエラーコード{}で終了しました",
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

/// 単一URLモード
fn download_single(ytdlp_path: &Path, url: &str, config: &DownloadConfig) -> Result<()> {
    if !config.quiet {
        println!("=== 単一URLモード ===\n");
    }
    download_url(ytdlp_path, url, config)
}

/// バッチモード
fn download_batch(ytdlp_path: &Path, urls: &[String], config: &DownloadConfig) -> Result<()> {
    if !config.quiet {
        println!("=== バッチモード ({} URLs) ===\n", urls.len());
    }

    for (i, url) in urls.iter().enumerate() {
        if !config.quiet {
            println!("[{}/{}] ダウンロード中...", i + 1, urls.len());
        }
        if let Err(e) = download_url(ytdlp_path, url, config) {
            eprintln!("エラー: {}", e);
            if !config.quiet {
                println!("次のURLに進みます...\n");
            }
        }
    }

    if !config.quiet {
        println!("すべてのダウンロードが完了しました。");
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
        println!("=== yt-dlp Video Downloader v1.3.2 ===\n");
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
