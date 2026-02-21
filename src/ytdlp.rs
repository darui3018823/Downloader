use crate::config::DownloadConfig;
use crate::platform::Platform;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn ensure_ytdlp(force_update: bool) -> Result<PathBuf> {
    let binaries_dir = PathBuf::from("./binaries");
    let ytdlp_path = binaries_dir.join(if cfg!(windows) {
        "yt-dlp.exe"
    } else {
        "yt-dlp"
    });

    if force_update && ytdlp_path.exists() {
        println!("既存のyt-dlpを削除しています...");
        fs::remove_file(&ytdlp_path).context("既存ファイルの削除に失敗しました")?;
    }

    if !force_update {
        if let Ok(output) = Command::new("yt-dlp").arg("--version").output() {
            if output.status.success() {
                println!("✓ 環境からyt-dlpを検出しました");
                return Ok(PathBuf::from("yt-dlp"));
            }
        }
    }

    if ytdlp_path.exists() && !force_update {
        println!("✓ {}からyt-dlpを検出しました", ytdlp_path.display());
        return Ok(ytdlp_path);
    }

    if force_update {
        println!("yt-dlpを最新バージョンに更新しています...");
    } else {
        println!("yt-dlpが見つかりません。GitHubからダウンロードしています...");
    }
    download_ytdlp(&binaries_dir, &ytdlp_path)?;

    Ok(ytdlp_path)
}

fn download_ytdlp(binaries_dir: &Path, ytdlp_path: &Path) -> Result<()> {
    fs::create_dir_all(binaries_dir).context("binariesディレクトリの作成に失敗しました")?;

    let download_url = if cfg!(windows) {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
    } else if cfg!(target_os = "macos") {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
    } else {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp"
    };

    println!("ダウンロード中: {}", download_url);

    let response =
        reqwest::blocking::get(download_url).context("yt-dlpのダウンロードに失敗しました")?;

    if !response.status().is_success() {
        bail!("ダウンロードエラー: ステータスコード {}", response.status());
    }

    let bytes = response
        .bytes()
        .context("レスポンスの読み取りに失敗しました")?;

    fs::write(ytdlp_path, &bytes).context("yt-dlpの保存に失敗しました")?;

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

/// yt-dlp の基本コマンドを構築
pub fn build_command(
    ytdlp_path: &Path,
    platform: Platform,
    url: &str,
    config: &DownloadConfig,
    is_live: bool,
) -> Command {
    let mut cmd = Command::new(ytdlp_path);

    // JS ランタイムの自動検出と適用
    if which::which("node").is_ok() {
        cmd.args(["--js-runtimes", "node"]);
    } else if which::which("bun").is_ok() {
        cmd.args(["--js-runtimes", "bun"]);
    } else if which::which("deno").is_ok() {
        cmd.args(["--js-runtimes", "deno"]);
    }

    if let Err(e) = fs::create_dir_all(&config.output_dir) {
        eprintln!("警告: 出力ディレクトリの作成に失敗: {}", e);
    }

    let output_template = format!("{}/%(title)s.%(ext)s", config.output_dir);

    if config.audio_only {
        cmd.args(["-x", "--audio-format", "mp3"]);
        cmd.args(["--output", &output_template, url]);

        if config.verbose {
            cmd.arg("--verbose");
        } else if config.quiet {
            cmd.arg("--quiet");
        }

        return cmd;
    }

    let format_arg = if let Some(quality) = &config.quality {
        match quality.as_str() {
            "best" => "bestvideo+bestaudio",
            q => q, // 1080p, 720p, etc.
        }
    } else if config.mp4_compat {
        match platform {
            Platform::Twitch => "1080p60+bestaudio",
            Platform::SoundCloud => "bestaudio/best",
            _ => "bestvideo[vcodec^=avc]+bestaudio[acodec^=mp4a]/bestvideo+bestaudio/best",
        }
    } else {
        match platform {
            Platform::Twitch => "1080p60+bestaudio",
            Platform::SoundCloud => "bestaudio/best",
            _ => "bestvideo+bestaudio/best",
        }
    };

    cmd.args(["-f", format_arg]);
    if !matches!(platform, Platform::SoundCloud) {
        if config.mp4_compat {
            cmd.args(["--merge-output-format", "mp4"]);
        } else {
            cmd.args(["--merge-output-format", &config.format]);
        }
    }

    if !config.no_metadata {
        cmd.args(["--embed-thumbnail", "--add-metadata"]);
    }

    if let Some(ref cookies) = config.cookies {
        cmd.args(["--cookies-from-browser", cookies]);
    }

    if !config.playlist {
        cmd.arg("--no-playlist");
    }

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

    if config.verbose {
        cmd.arg("--verbose");
    } else if config.quiet {
        cmd.arg("--quiet");
    }

    cmd.args(["--output", &output_template, url]);
    cmd
}

pub fn execute_download_command(mut cmd: Command, suppress_ytdlp_output: bool) -> Result<()> {
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
