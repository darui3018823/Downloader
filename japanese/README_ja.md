# Video Downloader (Rust版)

> 📖 **言語:** [English](../README.md) | [日本語](./README_ja.md)

Pythonの`downloader.py`をRustで書き直した動画ダウンローダーです。yt-dlpを使用して、複数のプラットフォームから動画をダウンロードします。

## 特徴

- 🚀 **自動yt-dlpダウンロード**: yt-dlpがシステムにインストールされていない場合、自動的にGitHub Releasesからダウンロードして`./binaries/`に保存します
- 🎯 **プラットフォーム自動検出**: URL から Twitch、YouTube、Twitter/X、niconico、SoundCloud、Instagram、TikTok、bilibili を自動検出し、最適な設定でダウンロード
- 🔄 **3つの動作モード**: 対話的ループモード、単一URLモード、バッチモード
- ⚙️ **詳細なカスタマイズ**: 出力先、画質、フォーマット、音声抽出、字幕指定などのオプション
- 🍪 **クッキー対応**: ブラウザクッキー認証に対応
- 📦 **単一実行ファイル**: Rustでコンパイルされた実行ファイル1つで動作
- ⚡ **高速・軽量**: Rustの高パフォーマンス

## サポートプラットフォーム

- **YouTube** (youtube.com, youtu.be)
  - Chromeクッキー認証
  - 最高画質 (bestvideo+bestaudio)
  - サムネイル・メタデータ埋め込み
  - 日本からのアクセスとして処理

- **Twitch** (twitch.tv)
  - 1080p60での保存
  - サムネイル・メタデータ埋め込み

- **Twitter/X** (twitter.com, x.com)
  - MP4形式で保存
  - サムネイル・メタデータ埋め込み

- **niconico** (nicovideo.jp, nico.ms)
  - 最高画質優先 (`bestvideo+bestaudio/best`)
  - 日本リージョンバイパス設定

- **SoundCloud** (soundcloud.com)
  - 音声優先の画質選択 (`bestaudio/best`)

- **Instagram** (instagram.com)
  - ブラウザ相当User-Agentで最適化

- **TikTok** (tiktok.com)
  - ブラウザ相当User-Agentで最適化

- **bilibili** (bilibili.com, b23.tv)
  - 汎用高画質設定 (`bv*+ba/b`)

- **その他のサイト**
  - 汎用設定で対応
  - 最高画質優先 (`bv*+ba/b`)
  - 字幕はオプション指定時のみダウンロード
  - Chromeクッキー認証（デフォルト）

## インストール

### ビルド済みバイナリを使用する場合

```bash
# Releaseビルド
cargo build --release

# 実行ファイルは target/release/downloader.exe に生成されます
```

### ソースからビルドする場合

```bash
# リポジリをクローン
git clone <repository-url>
cd Downloader

# Releaseビルド
cargo build --release
```

## 使い方

### モード1: 対話的ループモード (デフォルト)

引数なしで起動すると、複数のURLを連続してダウンロードできます。

```bash
.\target\release\downloader.exe

# URLを連続して入力
URL> https://www.youtube.com/watch?v=...
URL> https://www.twitch.tv/videos/...
URL> exit  # または quit、Ctrl+C で終了
```

**終了方法:**
- `exit` または `quit` と入力
- Ctrl+C で強制終了
- Ctrl+Z (Windows) または Ctrl+D (Unix) でEOF

### モード2: 単一URLモード

1つのURLをダウンロードして終了します。

```bash
.\target\release\downloader.exe --url "https://www.youtube.com/watch?v=..."
```

### 実験機能: Rustダウンロードモード (`--rust-download`)

yt-dlp には抽出だけを行わせ、実際のファイルダウンロードは Rust 側で実行します。

```bash
.\target\release\downloader.exe --url "https://www.youtube.com/watch?v=..." --rust-download
```

- `--rust-download` は `--url`（単一URLモード）専用です
- このモードでは通常の yt-dlp ダウンロードへの自動フォールバックはしません
- ハング/失敗した場合は `--rust-download` を外して再実行してください
- 詳細ログは `%USERPROFILE%/downloader/errorlog/*.log` に出力されます

### モード3: バッチモード

複数のURLを一度にダウンロードします。

```bash
.\target\release\downloader.exe --urls "https://youtube.com/..." "https://twitch.tv/..." "https://x.com/..."
```

###ヘルプ表示

```bash
.\target\release\downloader.exe --help
```

### バイナリ自己更新 (`-u` / `--update`)

downloader本体をGitHub Releasesの最新バイナリへ更新します。

```bash
.\target\release\downloader.exe -u
# または
.\target\release\downloader.exe --update
```

## Changelog

リリースノートは [Changelog.md](../Changelog.md) に集約しています。

## yt-dlpについて

このプログラムは以下の優先順位でyt-dlpを探します:

1. **システムのPATH**: `yt-dlp`コマンドが利用可能な場合はそれを使用
2. **ローカルバイナリ**: `./binaries/yt-dlp.exe`が存在する場合はそれを使用
3. **自動ダウンロード**: 上記が見つからない場合、GitHub Releasesから自動ダウンロード

初回実行時にyt-dlpが見つからない場合、自動的にダウンロードされます。

## 出力先

ダウンロードされた動画は、プログラムを実行したディレクトリ(カレントディレクトリ)に保存されます。

ファイル名: `{動画タイトル}.{拡張子}`

## 依存関係

- [yt-dlp](https://github.com/yt-dlp/yt-dlp) - 動画ダウンロードツール (自動ダウンロード)
- Rust 1.70以上

## ライセンス

BSD-2-Clause
