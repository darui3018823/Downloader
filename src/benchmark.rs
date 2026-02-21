use crate::config::DownloadConfig;
use crate::download::download_url;
use crate::rust_download::download_single_rust;
use crate::utils::dev_println;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

#[allow(dead_code)]
struct BenchmarkResult {
    method: String,
    elapsed: Duration,
    file_size: u64,
    avg_speed: f64,
    success: bool,
    error_msg: Option<String>,
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2} GB/s", bytes_per_sec / (1024.0 * 1024.0 * 1024.0))
    } else if bytes_per_sec >= 1024.0 * 1024.0 {
        format!("{:.2} MB/s", bytes_per_sec / (1024.0 * 1024.0))
    } else if bytes_per_sec >= 1024.0 {
        format!("{:.2} KB/s", bytes_per_sec / 1024.0)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}

fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs_f64();
    if total_secs >= 60.0 {
        let mins = (total_secs / 60.0).floor() as u64;
        let secs = total_secs - (mins as f64 * 60.0);
        format!("{}m {:.2}s", mins, secs)
    } else {
        format!("{:.2}s", total_secs)
    }
}

fn get_dir_total_size(dir: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                total += fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            } else if path.is_dir() {
                total += get_dir_total_size(&path);
            }
        }
    }
    total
}

fn run_ytdlp_benchmark(
    ytdlp_path: &Path,
    url: &str,
    config: &DownloadConfig,
    temp_dir: &Path,
) -> BenchmarkResult {
    let mut bench_config = config.clone();
    bench_config.output_dir = temp_dir.to_string_lossy().to_string();
    bench_config.quiet = true;

    dev_println(config, "benchmark: yt-dlp ダウンロード開始...");
    println!("  [1/2] yt-dlp でダウンロード中...");

    let start = Instant::now();
    let result = download_url(ytdlp_path, url, &bench_config);
    let elapsed = start.elapsed();

    match result {
        Ok(()) => {
            let file_size = get_dir_total_size(temp_dir);
            let avg_speed = if elapsed.as_secs_f64() > 0.0 {
                file_size as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            };
            dev_println(
                config,
                &format!(
                    "benchmark: yt-dlp 完了 elapsed={:.2}s size={}",
                    elapsed.as_secs_f64(),
                    format_size(file_size)
                ),
            );
            BenchmarkResult {
                method: "yt-dlp".to_string(),
                elapsed,
                file_size,
                avg_speed,
                success: true,
                error_msg: None,
            }
        }
        Err(e) => {
            dev_println(config, &format!("benchmark: yt-dlp エラー: {}", e));
            BenchmarkResult {
                method: "yt-dlp".to_string(),
                elapsed,
                file_size: 0,
                avg_speed: 0.0,
                success: false,
                error_msg: Some(format!("{}", e)),
            }
        }
    }
}

fn run_rust_benchmark(
    ytdlp_path: &Path,
    url: &str,
    config: &DownloadConfig,
    temp_dir: &Path,
) -> BenchmarkResult {
    let mut bench_config = config.clone();
    bench_config.output_dir = temp_dir.to_string_lossy().to_string();
    bench_config.rust_download = true;
    bench_config.quiet = true;

    dev_println(config, "benchmark: Rust ダウンロード開始...");
    println!("  [2/2] Rust でダウンロード中...");

    let start = Instant::now();
    let result = download_single_rust(ytdlp_path, url, &bench_config);
    let elapsed = start.elapsed();

    match result {
        Ok(()) => {
            let file_size = get_dir_total_size(temp_dir);
            let avg_speed = if elapsed.as_secs_f64() > 0.0 {
                file_size as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            };
            dev_println(
                config,
                &format!(
                    "benchmark: Rust 完了 elapsed={:.2}s size={}",
                    elapsed.as_secs_f64(),
                    format_size(file_size)
                ),
            );
            BenchmarkResult {
                method: "Rust".to_string(),
                elapsed,
                file_size,
                avg_speed,
                success: true,
                error_msg: None,
            }
        }
        Err(e) => {
            dev_println(config, &format!("benchmark: Rust エラー: {}", e));
            BenchmarkResult {
                method: "Rust".to_string(),
                elapsed,
                file_size: 0,
                avg_speed: 0.0,
                success: false,
                error_msg: Some(format!("{}", e)),
            }
        }
    }
}

fn print_result_table(
    ytdlp: &BenchmarkResult,
    rust: &BenchmarkResult,
    url: &str,
    config: &DownloadConfig,
) {
    let time_ytdlp = format_duration(ytdlp.elapsed);
    let time_rust = format_duration(rust.elapsed);
    let size_ytdlp = format_size(ytdlp.file_size);
    let size_rust = format_size(rust.file_size);
    let speed_ytdlp = format_speed(ytdlp.avg_speed);
    let speed_rust = format_speed(rust.avg_speed);
    let status_ytdlp = if ytdlp.success {
        "✓ Success".to_string()
    } else {
        format!("✗ {}", ytdlp.error_msg.as_deref().unwrap_or("Failed"))
    };
    let status_rust = if rust.success {
        "✓ Success".to_string()
    } else {
        format!("✗ {}", rust.error_msg.as_deref().unwrap_or("Failed"))
    };

    // 差分計算
    let time_diff = if ytdlp.success && rust.success && ytdlp.elapsed.as_secs_f64() > 0.0 {
        let pct = (rust.elapsed.as_secs_f64() - ytdlp.elapsed.as_secs_f64())
            / ytdlp.elapsed.as_secs_f64()
            * 100.0;
        format!("{:+.1}%", pct)
    } else {
        "-".to_string()
    };
    let speed_diff = if ytdlp.success && rust.success && ytdlp.avg_speed > 0.0 {
        let pct = (rust.avg_speed - ytdlp.avg_speed) / ytdlp.avg_speed * 100.0;
        format!("{:+.1}%", pct)
    } else {
        "-".to_string()
    };

    // Rust tuning info
    let tuning = config.resolve_rust_tuning();

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║                       Benchmark Results                             ║");
    println!("╠══════════════════════════════════════════════════════════════════════╣");
    println!("║  URL: {:<60} ║", truncate_str(url, 60));
    println!(
        "║  Rust tuning: chunk={}MB workers={} threads={}{:<17}║",
        tuning.chunk_size_bytes / (1024 * 1024),
        tuning.chunk_workers,
        tuning.runtime_threads,
        if config.rust_max_perf {
            " (max-perf)"
        } else {
            ""
        },
    );
    println!("╠═══════════════╦════════════╦════════════╦═══════════╦══════════════╣");
    println!("║ Method        ║ Time       ║ Size       ║ Speed     ║ Status       ║");
    println!("╠═══════════════╬════════════╬════════════╬═══════════╬══════════════╣");
    println!(
        "║ {:<13} ║ {:>10} ║ {:>10} ║ {:>9} ║ {:<12} ║",
        "yt-dlp",
        time_ytdlp,
        size_ytdlp,
        speed_ytdlp,
        truncate_str(&status_ytdlp, 12)
    );
    println!(
        "║ {:<13} ║ {:>10} ║ {:>10} ║ {:>9} ║ {:<12} ║",
        "Rust",
        time_rust,
        size_rust,
        speed_rust,
        truncate_str(&status_rust, 12)
    );
    println!("╠═══════════════╬════════════╬════════════╬═══════════╬══════════════╣");
    println!(
        "║ {:<13} ║ {:>10} ║ {:>10} ║ {:>9} ║              ║",
        "Rust vs yt-dlp", time_diff, "-", speed_diff
    );
    println!("╚═══════════════╩════════════╩════════════╩═══════════╩══════════════╝");
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

pub fn run_benchmark(ytdlp_path: &Path, url: &str, config: &DownloadConfig) -> Result<()> {
    println!("\n=== Benchmark: yt-dlp vs Rust ===\n");

    // 一時ディレクトリ作成
    let base_temp = std::env::temp_dir().join("downloader_benchmark");
    let temp_ytdlp = base_temp.join("ytdlp");
    let temp_rust = base_temp.join("rust");

    // クリーンアップ（前回の残りがあれば）
    let _ = fs::remove_dir_all(&base_temp);
    fs::create_dir_all(&temp_ytdlp).context("yt-dlp用一時ディレクトリの作成に失敗")?;
    fs::create_dir_all(&temp_rust).context("Rust用一時ディレクトリの作成に失敗")?;

    // 1. yt-dlp ベンチマーク
    let ytdlp_result = run_ytdlp_benchmark(ytdlp_path, url, config, &temp_ytdlp);

    // 2. Rust ベンチマーク
    let rust_result = run_rust_benchmark(ytdlp_path, url, config, &temp_rust);

    // 3. テーブル表示
    print_result_table(&ytdlp_result, &rust_result, url, config);

    // 4. クリーンアップ
    dev_println(
        config,
        &format!("benchmark: 一時ディレクトリ削除: {}", base_temp.display()),
    );
    let _ = fs::remove_dir_all(&base_temp);

    println!();
    Ok(())
}
