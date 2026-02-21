mod cli;
mod config;
mod download;
mod platform;
mod progress;
mod rust_download;
mod update;
mod utils;
mod ytdlp;

use anyhow::{bail, Result};
use clap::Parser;

use crate::cli::Cli;
use crate::config::DownloadConfig;
use crate::download::{download_batch, download_single, interactive_loop};
use crate::update::{show_credits, update_release_binary};
use crate::ytdlp::ensure_ytdlp;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let has_rust_perf_flags = cli.rust_max_perf
        || cli.rust_chunk_mb.is_some()
        || cli.rust_chunk_workers.is_some()
        || cli.rust_runtime_threads.is_some();

    if cli.update && cli.update_ytdlp {
        bail!("--update と --update-ytdlp は同時に指定できません");
    }

    if has_rust_perf_flags && !cli.rust_download {
        bail!(
            "--rust-chunk-mb / --rust-chunk-workers / --rust-runtime-threads / --rust-max-perf は --rust-download と一緒に指定してください"
        );
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
        println!("=== yt-dlp Video Downloader v2-rc-1 ===\n");
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
