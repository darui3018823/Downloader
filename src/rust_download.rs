use crate::config::{DownloadConfig, RustDownloadTuning};
use crate::progress::{make_download_progress_bar, make_phase_spinner, progress_style_known};
use crate::utils::{append_log, dev_println, new_error_log_path, sanitize_file_name};
use anyhow::{bail, Context, Result};
use indicatif::{MultiProgress, ProgressBar};
use reqwest::header::RANGE;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

#[derive(Debug, Clone)]
pub struct RustMediaStream {
    pub media_url: String,
    pub ext: String,
    pub protocol: String,
    pub headers: Vec<(String, String)>,
    pub filesize: Option<u64>,
    pub filesize_approx: Option<u64>,
    pub format_id: Option<String>,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    pub has_video: bool,
    pub has_audio: bool,
    pub has_fragments: bool,
    pub score: f64,
}

#[derive(Debug)]
struct RustDownloadCandidate {
    title: String,
    output_ext: String,
    single_stream: Option<RustMediaStream>,
    video_stream: Option<RustMediaStream>,
    audio_stream: Option<RustMediaStream>,
}

fn is_stream_protocol(protocol: &str) -> bool {
    let lower = protocol.to_ascii_lowercase();
    lower.contains("m3u8")
        || lower.contains("dash")
        || lower.contains("hls")
        || lower.contains("fragment")
}

fn value_to_u64(v: &Value) -> Option<u64> {
    v.as_u64().or_else(|| v.as_f64().map(|n| n as u64))
}

fn headers_from_value(value: &Value) -> Vec<(String, String)> {
    let mut headers = Vec::new();
    if let Some(map) = value.get("http_headers").and_then(Value::as_object) {
        for (key, val) in map {
            if let Some(text) = val.as_str() {
                headers.push((key.clone(), text.to_string()));
            }
        }
    }
    headers
}

fn merge_http_headers(
    base_headers: &[(String, String)],
    override_headers: &[(String, String)],
) -> Vec<(String, String)> {
    let mut merged = base_headers.to_vec();

    for (key, value) in override_headers {
        if let Some(index) = merged
            .iter()
            .position(|(existing, _)| existing.eq_ignore_ascii_case(key))
        {
            merged[index] = (key.clone(), value.clone());
        } else {
            merged.push((key.clone(), value.clone()));
        }
    }

    merged
}

fn stream_from_value(value: &Value, base_headers: &[(String, String)]) -> Option<RustMediaStream> {
    let media_url = value.get("url")?.as_str()?.to_string();
    if media_url.is_empty() {
        return None;
    }

    let protocol = value
        .get("protocol")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let has_fragments = value
        .get("fragments")
        .and_then(Value::as_array)
        .map(|arr| !arr.is_empty())
        .unwrap_or(false);

    let vcodec = value
        .get("vcodec")
        .and_then(Value::as_str)
        .map(|s| s.to_string());
    let acodec = value
        .get("acodec")
        .and_then(Value::as_str)
        .map(|s| s.to_string());

    let has_video = vcodec.as_deref().unwrap_or("none") != "none";
    let has_audio = acodec.as_deref().unwrap_or("none") != "none";

    let ext = value
        .get("ext")
        .and_then(Value::as_str)
        .unwrap_or("bin")
        .to_string();
    let format_id = value
        .get("format_id")
        .and_then(Value::as_str)
        .map(|s| s.to_string());

    let stream_headers = headers_from_value(value);
    let headers = merge_http_headers(base_headers, &stream_headers);

    let filesize = value.get("filesize").and_then(value_to_u64);
    let filesize_approx = value.get("filesize_approx").and_then(value_to_u64);

    let score = value
        .get("tbr")
        .and_then(Value::as_f64)
        .or_else(|| value.get("abr").and_then(Value::as_f64))
        .or_else(|| value.get("vbr").and_then(Value::as_f64))
        .unwrap_or(0.0);

    Some(RustMediaStream {
        media_url,
        ext,
        protocol,
        headers,
        filesize,
        filesize_approx,
        format_id,
        vcodec,
        acodec,
        has_video,
        has_audio,
        has_fragments,
        score,
    })
}

fn is_usable_stream(stream: &RustMediaStream) -> bool {
    !stream.media_url.is_empty() && !stream.has_fragments && !is_stream_protocol(&stream.protocol)
}

fn best_stream<'a, F>(streams: &'a [RustMediaStream], predicate: F) -> Option<&'a RustMediaStream>
where
    F: Fn(&RustMediaStream) -> bool,
{
    streams
        .iter()
        .filter(|s| predicate(s))
        .max_by(|a, b| a.score.total_cmp(&b.score))
}

fn extract_candidate_from_json(
    metadata: &Value,
    config: &DownloadConfig,
) -> Result<RustDownloadCandidate> {
    let title = metadata
        .get("title")
        .and_then(Value::as_str)
        .map(sanitize_file_name)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            metadata
                .get("id")
                .and_then(Value::as_str)
                .map(sanitize_file_name)
                .unwrap_or_else(|| "download".to_string())
        });

    let mut streams = Vec::new();
    let base_headers = headers_from_value(metadata);

    if let Some(requested_formats) = metadata.get("requested_formats").and_then(Value::as_array) {
        for value in requested_formats {
            if let Some(stream) = stream_from_value(value, &base_headers) {
                streams.push(stream);
            }
        }
    }

    if streams.is_empty() {
        if let Some(formats) = metadata.get("formats").and_then(Value::as_array) {
            for value in formats {
                if let Some(stream) = stream_from_value(value, &base_headers) {
                    streams.push(stream);
                }
            }
        }
    }

    if streams.is_empty() {
        if let Some(single) = stream_from_value(metadata, &base_headers) {
            streams.push(single);
        }
    }

    if streams.is_empty() {
        bail!("抽出JSONに直リンクURLがありません");
    }

    if config.audio_only {
        let selected = best_stream(&streams, |s| {
            is_usable_stream(s) && s.has_audio && !s.has_video
        })
        .or_else(|| best_stream(&streams, |s| is_usable_stream(s) && s.has_audio))
        .cloned()
        .context("Rust audio-onlyモードで利用可能な音声ストリームが見つかりません")?;

        return Ok(RustDownloadCandidate {
            title,
            output_ext: selected.ext.clone(),
            single_stream: Some(selected),
            video_stream: None,
            audio_stream: None,
        });
    }

    let video_stream = best_stream(&streams, |s| {
        is_usable_stream(s) && s.has_video && !s.has_audio
    })
    .cloned();
    let audio_stream = best_stream(&streams, |s| {
        is_usable_stream(s) && s.has_audio && !s.has_video
    })
    .cloned();

    if let (Some(video), Some(audio)) = (video_stream, audio_stream) {
        return Ok(RustDownloadCandidate {
            title,
            output_ext: config.format.clone(),
            single_stream: None,
            video_stream: Some(video),
            audio_stream: Some(audio),
        });
    }

    let single_stream = best_stream(&streams, |s| {
        is_usable_stream(s) && s.has_video && s.has_audio
    })
    .or_else(|| best_stream(&streams, is_usable_stream))
    .cloned()
    .context("Rustモードで利用可能な単一ストリームが見つかりません")?;

    Ok(RustDownloadCandidate {
        title,
        output_ext: single_stream.ext.clone(),
        single_stream: Some(single_stream),
        video_stream: None,
        audio_stream: None,
    })
}

fn extract_with_ytdlp(
    ytdlp_path: &Path,
    url: &str,
    config: &DownloadConfig,
    log_path: &Path,
) -> Result<Value> {
    let mut cmd = Command::new(ytdlp_path);
    cmd.args(["-J", "--no-playlist"]);

    if config.audio_only {
        cmd.args(["-f", "bestaudio/best"]);
    } else {
        cmd.args(["-f", "best/bv*+ba/b"]);
    }

    if let Some(cookies) = &config.cookies {
        cmd.args(["--cookies-from-browser", cookies]);
    }

    cmd.arg(url);

    append_log(log_path, &format!("[extract] command: {:?}", cmd));
    let output = cmd.output().context("yt-dlp抽出の実行に失敗しました")?;

    append_log(
        log_path,
        &format!("[extract] exit: {:?}", output.status.code()),
    );
    append_log(
        log_path,
        &format!(
            "[extract] stderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ),
    );

    if !output.status.success() {
        bail!(
            "yt-dlp抽出が失敗しました。詳細はログを確認してください: {}",
            log_path.display()
        );
    }

    let stdout = String::from_utf8(output.stdout).context("抽出JSONのUTF-8変換に失敗しました")?;
    append_log(log_path, &format!("[extract] stdout_len: {}", stdout.len()));

    let metadata: Value =
        serde_json::from_str(&stdout).context("yt-dlp抽出JSONの解析に失敗しました")?;
    append_log(
        log_path,
        &format!(
            "[extract] json:\n{}",
            serde_json::to_string_pretty(&metadata)
                .unwrap_or_else(|_| "<json pretty print failed>".to_string())
        ),
    );

    Ok(metadata)
}

async fn download_range_chunk(
    client: Arc<reqwest::Client>,
    stream: RustMediaStream,
    output_path: PathBuf,
    log_path: PathBuf,
    start: u64,
    end: u64,
) -> Result<u64> {
    let range_header = format!("bytes={}-{}", start, end);

    let mut request = client.get(&stream.media_url).header(RANGE, range_header);
    for (key, value) in &stream.headers {
        request = request.header(key, value);
    }

    let response = request
        .send()
        .await
        .context("Rangeリクエスト送信に失敗しました")?;
    let status = response.status();
    if !(status == reqwest::StatusCode::PARTIAL_CONTENT
        || (status == reqwest::StatusCode::OK && start == 0))
    {
        bail!("Range取得に失敗しました: {}", status);
    }
    if status == reqwest::StatusCode::OK && start > 0 {
        bail!("サーバーがRangeヘッダを無視しました");
    }

    let bytes = response
        .bytes()
        .await
        .context("Rangeレスポンス読み取りに失敗しました")?;
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .open(&output_path)
        .await
        .with_context(|| format!("出力ファイルを開けません: {}", output_path.display()))?;
    file.seek(std::io::SeekFrom::Start(start))
        .await
        .context("Range書き込み位置のシークに失敗しました")?;
    file.write_all(&bytes)
        .await
        .context("Rangeデータの書き込みに失敗しました")?;

    append_log(
        &log_path,
        &format!(
            "[download] chunk_done start={} end={} size={}",
            start,
            end,
            bytes.len()
        ),
    );

    Ok(bytes.len() as u64)
}

async fn rust_download_stream(
    stream: &RustMediaStream,
    output_path: &Path,
    log_path: &Path,
    progress_bar: ProgressBar,
    tuning: RustDownloadTuning,
    dev: bool,
) -> Result<()> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(300))
        .build()
        .context("HTTPクライアントの作成に失敗しました")?;
    let client = Arc::new(client);

    let mut head_request = client.head(&stream.media_url);
    for (key, value) in &stream.headers {
        head_request = head_request.header(key, value);
    }

    let head_response = head_request
        .send()
        .await
        .context("HEADリクエストに失敗しました")?;
    let head_content_length = head_response.content_length().filter(|size| *size > 0);
    let json_content_length = stream.filesize.filter(|size| *size > 0).or_else(|| {
        stream
            .filesize_approx
            .filter(|size| *size > 0)
            .map(|size| size as u64)
    });
    let total_size = head_content_length.or(json_content_length);
    let accept_ranges = head_response
        .headers()
        .get("accept-ranges")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_ascii_lowercase().contains("bytes"))
        .unwrap_or(false);

    append_log(
        log_path,
        &format!("[download] format_id: {:?}", stream.format_id),
    );
    append_log(log_path, &format!("[download] vcodec: {:?}", stream.vcodec));
    append_log(log_path, &format!("[download] acodec: {:?}", stream.acodec));
    append_log(log_path, &format!("[download] url: {}", stream.media_url));
    append_log(
        log_path,
        &format!("[download] protocol: {}", stream.protocol),
    );
    append_log(
        log_path,
        &format!("[download] output: {}", output_path.display()),
    );
    append_log(
        log_path,
        &format!("[download] headers_from_extract: {:?}", stream.headers),
    );

    append_log(
        log_path,
        &format!("[download] head_status: {}", head_response.status()),
    );
    append_log(
        log_path,
        &format!("[download] head_headers: {:?}", head_response.headers()),
    );

    append_log(
        log_path,
        &format!("[download] accept_ranges: {}", accept_ranges),
    );
    append_log(
        log_path,
        &format!("[download] total_size: {:?}", total_size),
    );
    append_log(
        log_path,
        &format!("[download] head_content_length: {:?}", head_content_length),
    );
    append_log(
        log_path,
        &format!("[download] json_filesize: {:?}", stream.filesize),
    );
    append_log(
        log_path,
        &format!("[download] json_filesize_approx: {:?}", stream.filesize_approx),
    );

    if let Some(total) = total_size {
        progress_bar.set_style(progress_style_known()?);
        progress_bar.disable_steady_tick();
        progress_bar.set_length(total);
    } else {
        progress_bar.set_message(format!("{} (size unknown)", progress_bar.message()));
    }

    if dev {
        println!(
            "[dev] stream={:?} accept_ranges={} head_size={:?} json_size={:?}/{:?} resolved_total={:?}",
            stream.format_id,
            accept_ranges,
            head_content_length,
            stream.filesize,
            stream.filesize_approx,
            total_size
        );
    }

    if head_response.status().is_success() && total_size.is_some() {
        let total = total_size.unwrap_or(0);
        let mut file = tokio::fs::File::create(output_path)
            .await
            .with_context(|| {
                format!("出力ファイル作成に失敗しました: {}", output_path.display())
            })?;
        file.set_len(total).await.with_context(|| {
            format!("出力ファイル拡張に失敗しました: {}", output_path.display())
        })?;
        file.flush()
            .await
            .context("初期ファイルflushに失敗しました")?;

        let chunk_size = tuning.chunk_size_bytes.max(1024 * 1024);
        let worker_count = tuning.chunk_workers.max(1);
        append_log(
            log_path,
            &format!(
                "[download] parallel_range enabled total={} chunk_size={} workers={}",
                total, chunk_size, worker_count
            ),
        );
        if dev {
            println!(
                "[dev] parallel_range enabled total={} chunk={} workers={}",
                total, chunk_size, worker_count
            );
        }
        let semaphore = Arc::new(tokio::sync::Semaphore::new(worker_count));
        let mut handles = Vec::new();

        let mut start = 0u64;
        while start < total {
            let end = (start + chunk_size - 1).min(total - 1);

            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .context("ダウンロードワーカ取得に失敗しました")?;
            let client_cloned = client.clone();
            let stream_cloned = stream.clone();
            let path_cloned = output_path.to_path_buf();
            let log_cloned = log_path.to_path_buf();
            let pb = progress_bar.clone();

            handles.push(tokio::spawn(async move {
                let _permit = permit;
                let written = download_range_chunk(
                    client_cloned,
                    stream_cloned,
                    path_cloned,
                    log_cloned,
                    start,
                    end,
                )
                .await?;
                pb.inc(written);
                Ok::<u64, anyhow::Error>(written)
            }));

            start = end + 1;
        }

        let mut written_total = 0u64;
        for handle in handles {
            let written = handle
                .await
                .map_err(|_| anyhow::anyhow!("チャンクDLタスクがpanicしました"))??;
            written_total += written;
        }

        append_log(
            log_path,
            &format!("[download] completed_bytes: {}", written_total),
        );
    } else {
        let mut request = client.get(&stream.media_url);
        for (key, value) in &stream.headers {
            request = request.header(key, value);
        }
        let response = request
            .send()
            .await
            .context("サイズ不明時のGET送信に失敗しました")?;
        if !response.status().is_success() {
            bail!("HTTPステータスが異常です: {}", response.status());
        }

        let mut response = response;
        if let Some(total) = response.content_length().filter(|size| *size > 0) {
            progress_bar.set_style(progress_style_known()?);
            progress_bar.disable_steady_tick();
            progress_bar.set_length(total);
        }

        let mut file = tokio::fs::File::create(output_path)
            .await
            .with_context(|| {
                format!("出力ファイル作成に失敗しました: {}", output_path.display())
            })?;
        let mut total_written = 0u64;

        while let Some(chunk) = response
            .chunk()
            .await
            .context("サイズ不明時のレスポンス読み取りに失敗しました")?
        {
            file.write_all(&chunk)
                .await
                .context("サイズ不明時の書き込みに失敗しました")?;
            total_written += chunk.len() as u64;
            progress_bar.inc(chunk.len() as u64);
        }

        if progress_bar.length().is_none() {
            progress_bar.set_length(total_written);
            progress_bar.set_position(total_written);
        }
        append_log(
            log_path,
            &format!("[download] completed_bytes: {}", total_written),
        );
    }

    progress_bar.finish_with_message(format!("{} done", progress_bar.message()));

    Ok(())
}

fn ffmpeg_merge_streams(
    video_path: &Path,
    audio_path: &Path,
    output_path: &Path,
    log_path: &Path,
) -> Result<()> {
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-hide_banner",
        "-y",
        "-i",
        &video_path.to_string_lossy(),
        "-i",
        &audio_path.to_string_lossy(),
        "-c",
        "copy",
        &output_path.to_string_lossy(),
    ]);

    append_log(log_path, &format!("[ffmpeg] command: {:?}", cmd));
    let output = cmd.output().context("ffmpeg結合の実行に失敗しました")?;
    append_log(
        log_path,
        &format!("[ffmpeg] exit: {:?}", output.status.code()),
    );
    append_log(
        log_path,
        &format!(
            "[ffmpeg] stderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ),
    );

    if !output.status.success() {
        bail!(
            "ffmpeg結合に失敗しました (code: {:?})",
            output.status.code()
        );
    }

    Ok(())
}

pub fn download_single_rust(ytdlp_path: &Path, url: &str, config: &DownloadConfig) -> Result<()> {
    let log_path = new_error_log_path(url)?;
    append_log(&log_path, "=== rust-download mode start ===");
    append_log(&log_path, &format!("[input] url: {}", url));
    append_log(&log_path, &format!("[config] {:?}", config));

    let extract_phase = make_phase_spinner(config.quiet)?;
    extract_phase.set_message("[flow] 1/4 抽出中...");

    let metadata = extract_with_ytdlp(ytdlp_path, url, config, &log_path)
        .with_context(|| format!("抽出処理に失敗しました。ログ: {}", log_path.display()))?;
    let candidate = extract_candidate_from_json(&metadata, config)
        .with_context(|| format!("抽出結果の解釈に失敗しました。ログ: {}", log_path.display()))?;
    let tuning = config.resolve_rust_tuning();

    append_log(&log_path, &format!("[tuning] {:?}", tuning));
    extract_phase.finish_and_clear();

    fs::create_dir_all(&config.output_dir).context("出力ディレクトリの作成に失敗しました")?;
    let file_name = format!("{}.{}", candidate.title, candidate.output_ext);
    let output_path = PathBuf::from(&config.output_dir).join(file_name);

    if !config.quiet {
        println!("Rust download mode: {}", output_path.display());
        println!("詳細ログ: {}", log_path.display());
        println!(
            "Rust tuning: chunk={}MB, chunk_workers={}, runtime_threads={}",
            tuning.chunk_size_bytes / (1024 * 1024),
            tuning.chunk_workers,
            tuning.runtime_threads
        );
    }

    dev_println(
        config,
        &format!(
            "candidate single={} video={} audio={}",
            candidate.single_stream.is_some(),
            candidate.video_stream.is_some(),
            candidate.audio_stream.is_some()
        ),
    );

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(tuning.runtime_threads)
        .enable_all()
        .build()
        .context("tokio runtimeの初期化に失敗しました")?;

    if let Some(single_stream) = &candidate.single_stream {
        let multi = MultiProgress::new();
        let single_pb = make_download_progress_bar(Some(&multi), "single", config.quiet)?;

        runtime
            .block_on(rust_download_stream(
                single_stream,
                &output_path,
                &log_path,
                single_pb,
                tuning,
                config.dev,
            ))
            .with_context(|| {
                format!(
                    "Rustダウンロード失敗。ハング/失敗時は --rust-download を外してください。ログ: {}",
                    log_path.display()
                )
            })?;
    } else if let (Some(video_stream), Some(audio_stream)) =
        (&candidate.video_stream, &candidate.audio_stream)
    {
        let temp_video = output_path.with_extension("video.tmp");
        let temp_audio = output_path.with_extension("audio.tmp");

        let multi = MultiProgress::new();
        let video_pb = make_download_progress_bar(Some(&multi), "video", config.quiet)?;
        let audio_pb = make_download_progress_bar(Some(&multi), "audio", config.quiet)?;

        let split_result = runtime.block_on(async {
            tokio::try_join!(
                rust_download_stream(
                    video_stream,
                    &temp_video,
                    &log_path,
                    video_pb,
                    tuning,
                    config.dev
                ),
                rust_download_stream(
                    audio_stream,
                    &temp_audio,
                    &log_path,
                    audio_pb,
                    tuning,
                    config.dev
                ),
            )
        });

        split_result.with_context(|| {
            format!(
                "動画/音声ストリームDL失敗。ハング/失敗時は --rust-download を外してください。ログ: {}",
                log_path.display()
            )
        })?;

        let merge_phase = make_phase_spinner(config.quiet)?;
        merge_phase.set_message("[flow] 3/4 ffmpeg結合中...");

        ffmpeg_merge_streams(&temp_video, &temp_audio, &output_path, &log_path).with_context(
            || {
                format!(
                    "ffmpeg結合失敗。ハング/失敗時は --rust-download を外してください。ログ: {}",
                    log_path.display()
                )
            },
        )?;
        merge_phase.finish_and_clear();

        let _ = fs::remove_file(&temp_video);
        let _ = fs::remove_file(&temp_audio);
    } else {
        bail!(
            "Rustダウンロード候補の解釈に失敗しました。ハング/失敗時は --rust-download を外してください"
        );
    }

    append_log(&log_path, "=== rust-download mode done ===");

    if !config.quiet {
        println!("[flow] 4/4 完了");
    }
    if !config.quiet {
        println!("\n✓ Rustダウンロードが完了しました。\n");
    }

    Ok(())
}
