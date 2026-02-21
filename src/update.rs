use crate::config::{REPO_NAME, REPO_OWNER};
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;
// ...existing code...
use std::time::Duration;

pub fn show_credits() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                 Video Downloader v2-rc-4                     ║");
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
    println!("║                v2-beta5 - Async range perf tuning            ║");
    println!("║                v2-beta6 - Robust range fallback              ║");
    println!("║                v2-beta7 - Max perf tuning options            ║");
    println!("║                v2-rc-1  - --dev flag, progress UI            ║");
    println!("║                v2-rc-2  - --benchmark, metadata embed        ║");
    println!("║                v2-rc-3  - yt-dlp tag compat, lang=jpn        ║");
    println!("║                v2-rc-4  - Unicode filename sanitize          ║");
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

pub fn select_latest_asset(assets: &[Value]) -> Option<(String, String)> {
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

pub fn update_release_binary() -> Result<()> {
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
