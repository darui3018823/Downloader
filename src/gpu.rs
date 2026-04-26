use anyhow::{bail, Context, Result};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuEncoder {
    Nvenc,
    NvencFallback,
    Amf,
    Cpu,
}

impl GpuEncoder {
    pub fn encoder_name(&self) -> &'static str {
        match self {
            GpuEncoder::Nvenc | GpuEncoder::NvencFallback => "hevc_nvenc",
            GpuEncoder::Amf => "hevc_amf",
            GpuEncoder::Cpu => "libx265",
        }
    }

    pub fn hwaccel_args(&self) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();

        match self {
            GpuEncoder::Nvenc => {
                args.extend([
                    "-hwaccel".into(),
                    "cuda".into(),
                    "-hwaccel_output_format".into(),
                    "cuda".into(),
                ]);
            }
            GpuEncoder::NvencFallback => {
                args.extend(["-hwaccel".into(), "auto".into()]);
            }
            GpuEncoder::Amf | GpuEncoder::Cpu => {}
        }
        args
    }

    pub fn build_encode_args(&self, ten_bit: bool) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();

        args.extend(["-c:v".into(), self.encoder_name().into()]);

        match self {
            GpuEncoder::Nvenc | GpuEncoder::NvencFallback => {
                args.extend([
                    "-preset".into(),
                    "p7".into(),
                    "-tune".into(),
                    "hq".into(),
                    "-rc".into(),
                    "vbr".into(),
                    "-cq".into(),
                    "24".into(),
                    "-b:v".into(),
                    "0".into(),
                ]);
                if ten_bit {
                    args.extend([
                        "-profile:v".into(),
                        "main10".into(),
                        "-pix_fmt".into(),
                        "p010le".into(),
                    ]);
                }
            }
            GpuEncoder::Amf => {
                args.extend([
                    "-quality".into(),
                    "quality".into(),
                    "-rc".into(),
                    "cqp".into(),
                    "-qp_i".into(),
                    "24".into(),
                    "-qp_p".into(),
                    "24".into(),
                ]);
                if ten_bit {
                    args.extend(["-pix_fmt".into(), "p010le".into()]);
                }
            }
            GpuEncoder::Cpu => {
                args.extend([
                    "-crf".into(),
                    "24".into(),
                    "-preset".into(),
                    "medium".into(),
                ]);
                if ten_bit {
                    args.extend([
                        "-profile:v".into(),
                        "main10".into(),
                        "-pix_fmt".into(),
                        "yuv420p10le".into(),
                    ]);
                }
            }
        }

        args.extend(["-c:a".into(), "aac".into(), "-b:a".into(), "192k".into()]);

        args
    }
}

/// 通常ダウンロードパス向けのシンプルな HEVC 変換。
/// 成功時は元ファイルを上書きして `Ok(true)`、ffmpeg 失敗時は `Ok(false)` を返す。
pub fn run_hevc_transcode(
    input: &Path,
    encoder: GpuEncoder,
    ten_bit: bool,
    quiet: bool,
) -> Result<bool> {
    let parent = input.parent().unwrap_or(Path::new("."));
    let stem = input
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let ext = input
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let tmp_name = format!("{}_hevc_tmp.{}", stem, ext);
    let tmp_path = parent.join(&tmp_name);

    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-hide_banner", "-y"]);

    for arg in encoder.hwaccel_args() {
        cmd.arg(arg);
    }

    cmd.args(["-i", &input.to_string_lossy()]);

    for arg in encoder.build_encode_args(ten_bit) {
        cmd.arg(arg);
    }

    cmd.arg(tmp_path.as_os_str());

    if quiet {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    }

    let status = cmd
        .status()
        .context("FFmpeg HEVC変換の起動に失敗しました")?;

    if !status.success() {
        let _ = fs::remove_file(&tmp_path);
        return Ok(false);
    }

    fs::rename(&tmp_path, input).context("HEVC変換後ファイルのリネームに失敗しました")?;
    Ok(true)
}

fn check_encoder_available(encoder_name: &str) -> bool {
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.contains(encoder_name)
        }
        Err(_) => false,
    }
}

pub fn detect_gpu_encoder(quiet: bool) -> Result<GpuEncoder> {
    if check_encoder_available("hevc_nvenc") {
        if !quiet {
            println!("✓ NVIDIA GPU検出: hevc_nvenc を使用します");
        }
        return Ok(GpuEncoder::Nvenc);
    }

    if check_encoder_available("hevc_amf") {
        if !quiet {
            println!("✓ AMD GPU検出: hevc_amf を使用します");
        }
        return Ok(GpuEncoder::Amf);
    }

    if quiet {
        return Ok(GpuEncoder::Cpu);
    }

    println!();
    println!("⚠ GPUエンコーダ (hevc_nvenc / hevc_amf) が検出されませんでした。");
    print!("  CPU (libx265) で続行しますか？ (y/N): ");
    io::stdout()
        .flush()
        .context("stdout flush に失敗しました")?;

    let stdin = io::stdin();
    let mut line = String::new();
    stdin
        .lock()
        .read_line(&mut line)
        .context("ユーザー入力の読み取りに失敗しました")?;

    let answer = line.trim().to_ascii_lowercase();
    if answer == "y" || answer == "yes" {
        println!("→ libx265 (CPU) で続行します");
        Ok(GpuEncoder::Cpu)
    } else {
        bail!("HEVC変換を中止しました（GPUエンコーダが利用できません）");
    }
}

pub fn try_encoder_fallback(
    original_encoder: GpuEncoder,
    quiet: bool,
) -> Result<Option<GpuEncoder>> {
    if original_encoder == GpuEncoder::Cpu {
        return Ok(None);
    }

    if original_encoder == GpuEncoder::Nvenc {
        if !quiet {
            println!();
            println!("⚠ GPUエンコード (完全ハードウェア処理) に失敗しました。");
            println!("→ 互換モード (-hwaccel auto) で hevc_nvenc を再試行します...");
        }
        return Ok(Some(GpuEncoder::NvencFallback));
    }

    if quiet {
        return Ok(Some(GpuEncoder::Cpu));
    }

    println!();
    println!(
        "⚠ GPUエンコード ({}) に失敗しました。",
        original_encoder.encoder_name()
    );
    print!("  CPU (libx265) でリトライしますか？ (y/N): ");
    io::stdout()
        .flush()
        .context("stdout flush に失敗しました")?;

    let stdin = io::stdin();
    let mut line = String::new();
    stdin
        .lock()
        .read_line(&mut line)
        .context("ユーザー入力の読み取りに失敗しました")?;

    let answer = line.trim().to_ascii_lowercase();
    if answer == "y" || answer == "yes" {
        println!("→ libx265 (CPU) でリトライします");
        Ok(Some(GpuEncoder::Cpu))
    } else {
        Ok(None)
    }
}
