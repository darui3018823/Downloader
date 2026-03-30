use crate::cli::Cli;
use std::thread;

pub const REPO_OWNER: &str = "darui3018823";
pub const REPO_NAME: &str = "Downloader";
pub const DEFAULT_RUST_CHUNK_SIZE_MB: u64 = 8;
pub const DEFAULT_RUST_CHUNK_WORKERS: usize = 6;
pub const DEFAULT_RUST_RUNTIME_THREADS: usize = 4;

/// ダウンロード設定
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    pub output_dir: String,
    pub audio_only: bool,
    pub quality: Option<String>,
    pub format: String,
    pub no_metadata: bool,
    pub cookies: Option<String>,
    pub playlist: bool,
    pub write_sub: bool,
    pub sub_lang: Option<String>,
    pub sub_format: Option<String>,
    pub convert_subs: Option<String>,
    pub verbose: bool,
    pub quiet: bool,
    pub dev: bool,
    pub threads: Option<usize>,
    pub rust_download: bool,
    pub rust_chunk_mb: Option<u64>,
    pub rust_chunk_workers: Option<usize>,
    pub rust_runtime_threads: Option<usize>,
    pub rust_normal_perf: bool,
    pub mp4_compat: bool,
    pub hevc: bool,
    pub ten_bit: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct RustDownloadTuning {
    pub chunk_size_bytes: u64,
    pub chunk_workers: usize,
    pub runtime_threads: usize,
}

impl DownloadConfig {
    pub fn from_cli(cli: &Cli) -> Self {
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
            dev: cli.dev,
            threads: cli.threads,
            rust_download: cli.rust_download,
            rust_chunk_mb: cli.rust_chunk_mb,
            rust_chunk_workers: cli.rust_chunk_workers,
            rust_runtime_threads: cli.rust_runtime_threads,
            rust_normal_perf: cli.rust_normal_perf,
            mp4_compat: cli.mp4_compat,
            hevc: cli.hevc,
            ten_bit: cli.ten_bit,
        }
    }

    pub fn resolve_rust_tuning(&self) -> RustDownloadTuning {
        let logical_cores = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .max(1);

        let max_perf_chunk_mb = 2;
        let max_perf_chunk_workers = (logical_cores * 4).max(8);
        let max_perf_runtime_threads = (logical_cores * 2).max(4);

        let chunk_mb = self.rust_chunk_mb.unwrap_or(if !self.rust_normal_perf {
            max_perf_chunk_mb
        } else {
            DEFAULT_RUST_CHUNK_SIZE_MB
        });
        let chunk_workers = self
            .rust_chunk_workers
            .unwrap_or(if !self.rust_normal_perf {
                max_perf_chunk_workers
            } else {
                DEFAULT_RUST_CHUNK_WORKERS
            });
        let runtime_threads = self
            .rust_runtime_threads
            .unwrap_or(if !self.rust_normal_perf {
                max_perf_runtime_threads
            } else {
                DEFAULT_RUST_RUNTIME_THREADS
            });

        RustDownloadTuning {
            chunk_size_bytes: chunk_mb.saturating_mul(1024 * 1024),
            chunk_workers: chunk_workers.max(1),
            runtime_threads: runtime_threads.max(1),
        }
    }
}
