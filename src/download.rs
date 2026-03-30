use crate::config::DownloadConfig;
use crate::platform::Platform;
use crate::rust_download::download_single_rust;
use crate::ytdlp::{build_command, execute_download_command};
use anyhow::Result;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;

pub fn download_url(ytdlp_path: &Path, url: &str, config: &DownloadConfig) -> Result<()> {
    if url.trim().is_empty() {
        return Ok(());
    }

    let platform = Platform::detect(url);

    if !config.quiet {
        println!("検出されたプラットフォーム: {:?}", platform);
    }

    let cmd = build_command(ytdlp_path, platform, url, config, false);

    if !config.quiet {
        println!("ダウンロードを開始します...\n");
    }

    execute_download_command(cmd, false)?;

    if !config.quiet {
        println!("\n✓ ダウンロードが完了しました。\n");
    }

    Ok(())
}

pub fn download_single(ytdlp_path: &Path, url: &str, config: &DownloadConfig) -> Result<()> {
    if !config.quiet {
        println!("=== 単一URLモード ===\n");
    }

    if config.rust_download {
        return download_single_rust(ytdlp_path, url, config);
    }

    download_url(ytdlp_path, url, config)
}

pub fn download_batch(ytdlp_path: &Path, urls: &[String], config: &DownloadConfig) -> Result<()> {
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

    let total_urls = urls.len();
    let urls_arc = Arc::new(urls.to_vec());
    let current_idx = Arc::new(AtomicUsize::new(0));
    let (tx, rx) = mpsc::channel();

    let mut handles = Vec::with_capacity(max_workers);

    for _ in 0..max_workers {
        let urls = Arc::clone(&urls_arc);
        let current_idx = Arc::clone(&current_idx);
        let tx = tx.clone();
        let ytdlp_path = ytdlp_path.to_path_buf();
        let config = config.clone();

        handles.push(thread::spawn(move || loop {
            let idx = current_idx.fetch_add(1, Ordering::SeqCst);
            if idx >= urls.len() {
                break;
            }

            let url = urls[idx].clone();
            if url.trim().is_empty() {
                let _ = tx.send((url, Ok(())));
                continue;
            }

            let platform = Platform::detect(&url);
            let cmd = build_command(&ytdlp_path, platform, &url, &config, false);
            let result = execute_download_command(cmd, true);
            let _ = tx.send((url, result));
        }));
    }

    drop(tx);

    for (url, result) in rx {
        match result {
            Ok(()) => {
                completed += 1;
                if !config.quiet {
                    println!("[{}/{}] 完了: {}", completed + failed, total_urls, url);
                }
            }
            Err(e) => {
                failed += 1;
                eprintln!("エラー ({}): {}", url, e);
            }
        }
    }

    for handle in handles {
        if handle.join().is_err() {
            failed += 1;
            eprintln!("エラー: ダウンロードスレッドがpanicしました");
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

pub fn interactive_loop(ytdlp_path: &Path, config: &DownloadConfig) -> Result<()> {
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

                if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                    if !config.quiet {
                        println!("終了します。");
                    }
                    break;
                }

                if input.is_empty() {
                    continue;
                }

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
                if !config.quiet {
                    println!("\n終了します。");
                }
                break;
            }
        }
    }

    Ok(())
}
