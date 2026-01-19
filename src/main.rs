use anyhow::{bail, Context, Result};
use clap::Parser;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// yt-dlpを使用した動画ダウンローダー
#[derive(Parser)]
#[command(name = "downloader")]
#[command(version = "1.1.0")]
#[command(about = "yt-dlpを使用した動画ダウンローダー", long_about = None)]
struct Cli {
    /// 単一URLをダウンロードして終了
    #[arg(long)]
    url: Option<String>,

    /// 複数のURLを一度にダウンロード
    #[arg(long, num_args = 1..)]
    urls: Option<Vec<String>>,
}

/// yt-dlpバイナリのパスを取得または自動ダウンロード
fn ensure_ytdlp() -> Result<PathBuf> {
    // まず環境のPATHからyt-dlpを探す
    if let Ok(output) = Command::new("yt-dlp").arg("--version").output() {
        if output.status.success() {
            println!("✓ 環境からyt-dlpを検出しました");
            return Ok(PathBuf::from("yt-dlp"));
        }
    }

    // PATHにない場合、./binaries/yt-dlp.exeを確認
    let binaries_dir = PathBuf::from("./binaries");
    let ytdlp_path = binaries_dir.join(if cfg!(windows) {
        "yt-dlp.exe"
    } else {
        "yt-dlp"
    });

    if ytdlp_path.exists() {
        println!("✓ {}からyt-dlpを検出しました", ytdlp_path.display());
        return Ok(ytdlp_path);
    }

    // どちらにもない場合、GitHubからダウンロード
    println!("yt-dlpが見つかりません。GitHubからダウンロードしています...");
    download_ytdlp(&binaries_dir, &ytdlp_path)?;

    Ok(ytdlp_path)
}

/// GitHubのReleasesからyt-dlpをダウンロード
fn download_ytdlp(binaries_dir: &Path, ytdlp_path: &Path) -> Result<()> {
    // binariesディレクトリを作成
    fs::create_dir_all(binaries_dir)
        .context("binariesディレクトリの作成に失敗しました")?;

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
    let response = reqwest::blocking::get(download_url)
        .context("yt-dlpのダウンロードに失敗しました")?;

    if !response.status().is_success() {
        bail!("ダウンロードエラー: ステータスコード {}", response.status());
    }

    let bytes = response.bytes().context("レスポンスの読み取りに失敗しました")?;

    // ファイルに書き込み
    fs::write(ytdlp_path, &bytes)
        .context("yt-dlpの保存に失敗しました")?;

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
    Generic,
}

impl Platform {
    fn detect(url: &str) -> Self {
        if url.contains("twitch.tv") {
            Platform::Twitch
        } else if url.contains("youtube.com") || url.contains("youtu.be") {
            Platform::YouTube
        } else if url.contains("twitter.com") || url.contains("x.com") {
            Platform::Twitter
        } else {
            Platform::Generic
        }
    }
}

/// プラットフォームに応じたyt-dlpコマンドを構築
fn build_command(ytdlp_path: &Path, platform: Platform, url: &str) -> Command {
    let mut cmd = Command::new(ytdlp_path);
    let download_dir = "./";

    match platform {
        Platform::Twitch => {
            cmd.args([
                "-f", "1080p60+bestaudio",
                "--merge-output-format", "mp4",
                "--embed-thumbnail",
                "--add-metadata",
                "--output", &format!("{}/%(title)s.%(ext)s", download_dir),
                url,
            ]);
        }
        Platform::YouTube => {
            cmd.args([
                "--cookies-from-browser", "firefox",
                "-4",
                "-f", "bestvideo+bestaudio",
                "--merge-output-format", "mp4",
                "--embed-thumbnail",
                "--add-metadata",
                "--geo-bypass-country", "JP",
                "--output", &format!("{}/%(title)s.%(ext)s", download_dir),
                url,
            ]);
        }
        Platform::Twitter => {
            cmd.args([
                "--merge-output-format", "mp4",
                "--embed-thumbnail",
                "--add-metadata",
                "--output", &format!("{}/%(title)s.%(ext)s", download_dir),
                url,
            ]);
        }
        Platform::Generic => {
            println!("最高画質での保存ができない場合があります。");
            cmd.args([
                "--merge-output-format", "mp4",
                "--output", &format!("{}/%(title)s.%(ext)s", download_dir),
                "--embed-thumbnail",
                "--add-metadata",
                "--geo-bypass-country", "JP",
                "-f", "bestvideo+bestaudio/best",
                "--no-playlist",
                "--cookies-from-browser", "firefox",
                "--user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.74 Safari/537.36",
                "--write-sub",
                "--sub-lang", "all",
                "--sub-format", "best",
                "--convert-subs", "srt",
                "--ignore-errors",
                url,
            ]);
        }
    }

    cmd
}

/// URLをダウンロード
fn download_url(ytdlp_path: &Path, url: &str) -> Result<()> {
    if url.trim().is_empty() {
        return Ok(());
    }

    // プラットフォームを検出
    let platform = Platform::detect(url);
    println!("検出されたプラットフォーム: {:?}", platform);

    // コマンドを構築して実行
    let mut cmd = build_command(ytdlp_path, platform, url);

    println!("ダウンロードを開始します...\n");
    let status = cmd.status().context("yt-dlpの実行に失敗しました")?;

    if status.success() {
        println!("\n✓ ダウンロードが完了しました。\n");
    } else {
        bail!("yt-dlpがエラーコード{}で終了しました", status.code().unwrap_or(-1));
    }

    Ok(())
}

/// 単一URLモード
fn download_single(ytdlp_path: &Path, url: &str) -> Result<()> {
    println!("=== 単一URLモード ===\n");
    download_url(ytdlp_path, url)
}

/// バッチモード
fn download_batch(ytdlp_path: &Path, urls: &[String]) -> Result<()> {
    println!("=== バッチモード ({} URLs) ===\n", urls.len());
    
    for (i, url) in urls.iter().enumerate() {
        println!("[{}/{}] ダウンロード中...", i + 1, urls.len());
        if let Err(e) = download_url(ytdlp_path, url) {
            eprintln!("エラー: {}", e);
            println!("次のURLに進みます...\n");
        }
    }
    
    println!("すべてのダウンロードが完了しました。");
    Ok(())
}

/// 対話的ループモード
fn interactive_loop(ytdlp_path: &Path) -> Result<()> {
    println!("=== 対話的モード ===");
    println!("URLを入力してください (exit/quit で終了, Ctrl+C でも終了可能)\n");

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    loop {
        print!("URL> ");
        io::stdout().flush()?;

        match lines.next() {
            Some(Ok(input)) => {
                let input = input.trim();

                // 終了コマンドチェック
                if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                    println!("終了します。");
                    break;
                }

                // 空行はスキップ
                if input.is_empty() {
                    continue;
                }

                // URLをダウンロード
                if let Err(e) = download_url(ytdlp_path, input) {
                    eprintln!("エラー: {}", e);
                    println!("次のURLを入力してください。\n");
                }
            }
            Some(Err(e)) => {
                eprintln!("入力エラー: {}", e);
                break;
            }
            None => {
                // EOF (Ctrl+D on Unix, Ctrl+Z on Windows) または Ctrl+C
                println!("\n終了します。");
                break;
            }
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("=== yt-dlp Video Downloader v1.1.0 ===\n");

    // yt-dlpの確保
    let ytdlp_path = ensure_ytdlp()?;
    println!();

    // モード判定と実行
    match (cli.url, cli.urls) {
        (Some(url), None) => {
            // 単一URLモード
            download_single(&ytdlp_path, &url)?;
        }
        (None, Some(urls)) => {
            // バッチモード
            download_batch(&ytdlp_path, &urls)?;
        }
        (None, None) => {
            // 対話的ループモード
            interactive_loop(&ytdlp_path)?;
        }
        (Some(_), Some(_)) => {
            bail!("--url と --urls を同時に指定できません");
        }
    }

    Ok(())
}
