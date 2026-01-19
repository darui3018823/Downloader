use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

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
        anyhow::bail!("ダウンロードエラー: ステータスコード {}", response.status());
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

fn main() -> Result<()> {
    println!("=== yt-dlp Video Downloader ===\n");

    // yt-dlpの確保
    let ytdlp_path = ensure_ytdlp()?;

    // URLの入力を受け付け
    print!("動画のURLを入力してください: ");
    io::stdout().flush()?;

    let mut url = String::new();
    io::stdin()
        .read_line(&mut url)
        .context("URLの読み取りに失敗しました")?;
    let url = url.trim();

    if url.is_empty() {
        anyhow::bail!("URLが入力されていません");
    }

    // プラットフォームを検出
    let platform = Platform::detect(url);
    println!("検出されたプラットフォーム: {:?}\n", platform);

    // コマンドを構築して実行
    let mut cmd = build_command(&ytdlp_path, platform, url);

    println!("ダウンロードを開始します...\n");
    let status = cmd.status().context("yt-dlpの実行に失敗しました")?;

    if status.success() {
        println!("\n✓ ダウンロードが完了しました。");
    } else {
        anyhow::bail!("yt-dlpがエラーコード{}で終了しました", status.code().unwrap_or(-1));
    }

    Ok(())
}
