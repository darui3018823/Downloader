use crate::config::DownloadConfig;
use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn sanitize_file_name(input: &str) -> String {
    let mut result = input
        .replace('/', "⧸")
        .replace('\\', "⧹")
        .replace('|', "⏐")
        .replace('<', "＜")
        .replace('>', "＞")
        .replace('?', "？")
        .replace('*', "＊")
        .replace(':', "：")
        .replace('"', "＂");

    result.retain(|c| !c.is_control());

    result = result.trim().to_string();
    if result.is_empty() {
        "download".to_string()
    } else {
        result
    }
}

pub fn error_log_dir() -> PathBuf {
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

pub fn new_error_log_path(url: &str) -> Result<PathBuf> {
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

pub fn append_log(log_path: &Path, message: &str) {
    let mut opts = OpenOptions::new();
    opts.create(true).append(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_SHARE_READ: u32 = 0x00000001;
        const FILE_SHARE_WRITE: u32 = 0x00000002;
        const FILE_SHARE_DELETE: u32 = 0x00000004;
        opts.share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE);
    }

    let mut file = match opts.open(log_path) {
        Ok(file) => file,
        Err(_) => return,
    };

    let _ = writeln!(file, "{}", message);
}

pub fn dev_println(config: &DownloadConfig, message: &str) {
    if config.dev && !config.quiet {
        println!("[dev] {}", message);
    }
}
