use anyhow::{bail, Context, Result};
use std::io::{self, BufRead, Write};
use std::process::Command;

/// GPU エンコーダの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuEncoder {
    /// NVIDIA NVENC (hevc_nvenc) + cuda hwaccel
    Nvenc,
    /// NVIDIA NVENC (hevc_nvenc) + auto fallback hwaccel
    NvencFallback,
    /// AMD AMF (hevc_amf)
    Amf,
    /// CPU ソフトウェア (libx265)
    Cpu,
}

impl GpuEncoder {
    /// FFmpeg エンコーダ名
    pub fn encoder_name(&self) -> &'static str {
        match self {
            GpuEncoder::Nvenc | GpuEncoder::NvencFallback => "hevc_nvenc",
            GpuEncoder::Amf => "hevc_amf",
            GpuEncoder::Cpu => "libx265",
        }
    }

    /// FFmpeg エンコード引数を構築
    pub fn build_encode_args(&self, ten_bit: bool) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();

        match self {
            GpuEncoder::Nvenc => {
                // NVENC: 最速の完全ハードウェア処理
                args.extend([
                    "-hwaccel".into(),
                    "cuda".into(),
                    "-hwaccel_output_format".into(),
                    "cuda".into(),
                ]);
            }
            GpuEncoder::NvencFallback => {
                // NVENC (互換): hwaccel auto で安全にデコード高速化
                args.extend(["-hwaccel".into(), "auto".into()]);
            }
            GpuEncoder::Amf | GpuEncoder::Cpu => {
                // AMF / CPU はhwaccel不要
            }
        }

        // 入力は呼び出し側で追加

        // エンコーダ指定
        args.extend(["-c:v".into(), self.encoder_name().into()]);

        // エンコーダ固有パラメータ
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

        // 音声: AAC 192k
        args.extend(["-c:a".into(), "aac".into(), "-b:a".into(), "192k".into()]);

        args
    }
}

/// ffmpeg -encoders の出力からエンコーダの有無を判定
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

/// GPU エンコーダを検出してフォールバック
pub fn detect_gpu_encoder(quiet: bool) -> Result<GpuEncoder> {
    // NVIDIA (hevc_nvenc) を優先チェック
    if check_encoder_available("hevc_nvenc") {
        if !quiet {
            println!("✓ NVIDIA GPU検出: hevc_nvenc を使用します");
        }
        return Ok(GpuEncoder::Nvenc);
    }

    // AMD (hevc_amf) チェック
    if check_encoder_available("hevc_amf") {
        if !quiet {
            println!("✓ AMD GPU検出: hevc_amf を使用します");
        }
        return Ok(GpuEncoder::Amf);
    }

    // GPU が見つからない場合の CPU フォールバック
    if quiet {
        // quiet モードでは自動的に CPU にフォールバック
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

/// GPU エンコーダでの HEVC 変換が失敗した場合にフォールバックを試みる
pub fn try_encoder_fallback(
    original_encoder: GpuEncoder,
    quiet: bool,
) -> Result<Option<GpuEncoder>> {
    if original_encoder == GpuEncoder::Cpu {
        // 既に CPU なのでフォールバック先なし
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
