use clap::Parser;

pub fn parse_u64_ge1(value: &str) -> std::result::Result<u64, String> {
    let parsed = value
        .parse::<u64>()
        .map_err(|_| "1以上の整数を指定してください".to_string())?;

    if parsed == 0 {
        return Err("1以上の整数を指定してください".to_string());
    }

    Ok(parsed)
}

pub fn parse_threads(value: &str) -> std::result::Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| "--threads には1以上の整数を指定してください".to_string())?;

    if parsed == 0 {
        return Err("--threads には1以上の整数を指定してください".to_string());
    }

    Ok(parsed)
}

/// yt-dlpを使用した動画ダウンローダー
#[derive(Parser)]
#[command(name = "downloader")]
#[command(version = "2.0.0-rc-5")]
#[command(about = "yt-dlpを使用した動画ダウンローダー", long_about = None)]
pub struct Cli {
    /// 単一URLをダウンロードして終了
    #[arg(long)]
    pub url: Option<String>,

    /// 複数のURLを一度にダウンロード
    #[arg(long, num_args = 1..)]
    pub urls: Option<Vec<String>>,

    /// ダウンロード先ディレクトリ
    #[arg(short = 'o', long, default_value = "./")]
    pub output_dir: String,

    /// 音声のみダウンロード（mp3形式）
    #[arg(short = 'a', long)]
    pub audio_only: bool,

    /// 画質指定 (best, 1080p, 720p, 480p, 360p)
    #[arg(long)]
    pub quality: Option<String>,

    /// 出力フォーマット (mp4, mkv, webm)
    #[arg(short = 'f', long)]
    pub format: Option<String>,

    /// サムネイル・メタデータの埋め込みをスキップ
    #[arg(long)]
    pub no_metadata: bool,

    /// クッキー元のブラウザ (chrome, firefox, edge, safari)
    /// 指定しない場合はクッキーを使用しません
    #[arg(long)]
    pub cookies: Option<String>,

    /// プレイリスト全体をダウンロード
    #[arg(long)]
    pub playlist: bool,

    /// 字幕をダウンロード
    #[arg(long)]
    pub write_sub: bool,

    /// 字幕言語 (例: ja,en,all)
    #[arg(long)]
    pub sub_lang: Option<String>,

    /// 字幕フォーマット (例: srt,vtt,best)
    #[arg(long)]
    pub sub_format: Option<String>,

    /// 字幕変換フォーマット (例: srt,vtt)
    #[arg(long)]
    pub convert_subs: Option<String>,

    /// アプリ本体を最新Releaseバイナリに更新
    #[arg(short = 'u', long)]
    pub update: bool,

    /// yt-dlpを最新バージョンに更新
    #[arg(long)]
    pub update_ytdlp: bool,

    /// 詳細ログを出力
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// 最小限の出力のみ
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// 開発者向け詳細ログをTerminalにも表示
    #[arg(long)]
    pub dev: bool,

    /// バッチモード時の最大スレッド数（--urls 専用）
    #[arg(short = 't', long, value_parser = parse_threads)]
    pub threads: Option<usize>,

    /// 抽出のみyt-dlpを使い、ダウンロードはRustで実行（--url 専用・実験的）
    #[arg(long)]
    pub rust_download: bool,

    /// Rustダウンロード時のチャンクサイズ（MB）
    #[arg(long, value_parser = parse_u64_ge1)]
    pub rust_chunk_mb: Option<u64>,

    /// Rustダウンロード時の並列チャンクワーカー数
    #[arg(long, value_parser = parse_threads)]
    pub rust_chunk_workers: Option<usize>,

    /// Rustダウンロード時のtokio worker thread数
    #[arg(long, value_parser = parse_threads)]
    pub rust_runtime_threads: Option<usize>,

    /// Rustダウンロードを全力設定で実行（CPU/並列を強める）
    #[arg(long)]
    pub rust_max_perf: bool,

    /// MP4互換モード (H.264/AAC) でダウンロード
    #[arg(long)]
    pub mp4_compat: bool,

    /// ダウンロード後にHEVC (H.265) に再エンコード（GPU加速対応）
    #[arg(long)]
    pub hevc: bool,

    /// HEVC 10-bit (p010le) 出力を有効化（--hevc 必須）
    #[arg(long = "10bit")]
    pub ten_bit: bool,

    /// クレジット情報を表示
    #[arg(long)]
    pub credit: bool,

    /// 同一URLでyt-dlp vs Rustの速度ベンチマーク（--url 専用）
    #[arg(long)]
    pub benchmark: bool,
}
