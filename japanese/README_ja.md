# Video Downloader (Rust版)

> 📖 **言語:** [English](../README.md) | [日本語](./README_ja.md)

Pythonの`downloader.py`をRustで書き直した動画ダウンローダーです。yt-dlpを使用して、複数のプラットフォームから動画をダウンロードします。

## 特徴

- 🚀 **自動yt-dlpダウンロード**: yt-dlpがシステムにインストールされていない場合、自動的にGitHub Releasesからダウンロードして`./binaries/`に保存します
- 🎯 **プラットフォーム自動検出**: URL から Twitch、YouTube、Twitter/X を自動検出し、最適な設定でダウンロード
- 🔄 **3つの動作モード**: 対話的ループモード、単一URLモード、バッチモード
- ⚙️ **詳細なカスタマイズ**: 出力先、画質、フォーマット、音声抽出など11種類のオプション
- 🍪 **Chromeクッキー対応**: デフォルトでChromeのクッキーを使用（v1.2.0~）
- 📦 **単一実行ファイル**: Rustでコンパイルされた実行ファイル1つで動作
- ⚡ **高速・軽量**: Rustの高パフォーマンス

## サポートプラットフォーム

- **YouTube** (youtube.com, youtu.be)
  - Chromeクッキー認証（デフォルト、v1.2.0~）
  - 最高画質 (bestvideo+bestaudio)
  - サムネイル・メタデータ埋め込み
  - 日本からのアクセスとして処理

- **Twitch** (twitch.tv)
  - 1080p60での保存
  - サムネイル・メタデータ埋め込み

- **Twitter/X** (twitter.com, x.com)
  - MP4形式で保存
  - サムネイル・メタデータ埋め込み

- **その他のサイト**
  - 汎用設定で対応
  - 字幕ダウンロード (SRT形式)
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

### モード3: バッチモード

複数のURLを一度にダウンロードします。

```bash
.\target\release\downloader.exe --urls "https://youtube.com/..." "https://twitch.tv/..." "https://x.com/..."
```

###ヘルプ表示

```bash
.\target\release\downloader.exe --help
```

## v1.2.0の新機能

### 高度なオプション

#### 出力ディレクトリ指定 (`-o` / `--output-dir`)

```bash
# downloadsフォルダに保存
.\target\release\downloader.exe --url "..." -o "./downloads"

# 音楽専用フォルダに保存
.\target\release\downloader.exe -o "./music" -a --url "..."
```

#### 音声のみダウンロード (`-a` / `--audio-only`)

音声のみをmp3形式でダウンロードします。

```bash
.\target\release\downloader.exe --url "..." --audio-only
# または短縮形
.\target\release\downloader.exe --url "..." -a
```

#### 画質指定 (`--quality`)

解像度を指定してダウンロードします。

```bash
# 720pでダウンロード
.\target\release\downloader.exe --url "..." --quality 720p

# 利用可能な画質: best (デフォルト), 1080p, 720p, 480p, 360p
```

#### 出力フォーマット指定 (`-f` / `--format`)

```bash
# MKV形式で保存
.\target\release\downloader.exe --url "..." -f mkv

# 利用可能な形式: mp4 (デフォルト), mkv, webm
```

#### メタデータスキップ (`--no-metadata`)

サムネイルとメタデータの埋め込みをスキップして高速化します。

```bash
.\target\release\downloader.exe --url "..." --no-metadata
```

#### クッキーブラウザ指定 (`--cookies`)

**v1.2.0からデフォルトがChromeに変更されました。**

```bash
# Firefoxのクッキーを使用
.\target\release\downloader.exe --url "..." --cookies firefox

# 対応ブラウザ: chrome (デフォルト), firefox, edge, safari
```

#### プレイリストダウンロード (`--playlist`)

プレイリスト全体をダウンロードします（デフォルトは単一動画）。

```bash
.\target\release\downloader.exe --url "..." --playlist
```

#### yt-dlp更新 (`--update-ytdlp`)

yt-dlpを最新バージョンに更新します。

```bash
.\target\release\downloader.exe --update-ytdlp
```

#### 詳細ログ (`-v` / `--verbose`)

詳細なログを出力してデバッグします。

```bash
.\target\release\downloader.exe --url "..." -v
```

#### 静寂モード (`-q` / `--quiet`)

最小限の出力のみ表示します。

```bash
.\target\release\downloader.exe --url "..." -q
```

#### クレジット表示 (`--credit`)

開発者情報とクレジットを表示します。

```bash
.\target\release\downloader.exe --credit
```

### 複合使用例

```bash
# 音声のみ、Firefoxクッキー、出力先指定
.\target\release\downloader.exe --url "..." -a --cookies firefox -o "./music"

# 720p、静寂モード、メタデータなし
.\target\release\downloader.exe --url "..." --quality 720p -q --no-metadata

# プレイリスト、詳細ログ、MKV形式
.\target\release\downloader.exe --url "..." --playlist -v -f mkv
```

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
