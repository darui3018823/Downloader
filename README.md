# Video Downloader (Rust版)

Pythonの`downloader.py`をRustで書き直した動画ダウンローダーです。yt-dlpを使用して、複数のプラットフォームから動画をダウンロードします。

## 特徴

- 🚀 **自動yt-dlpダウンロード**: yt-dlpがシステムにインストールされていない場合、自動的にGitHub Releasesからダウンロードして`./binaries/`に保存します
- 🎯 **プラットフォーム自動検出**: URL から Twitch、YouTube、Twitter/X を自動検出し、最適な設定でダウンロード
- 📦 **3つの動作モード**: 対話的ループモード、単一URLモード、バッチモード
- 📦 **単一実行ファイル**: Rustでコンパイルされた実行ファイル1つで動作
- ⚡ **高速・軽量**: Rustの高パフォーマンス

## サポートプラットフォーム

- **YouTube** (youtube.com, youtu.be)
  - Firefoxクッキー認証
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
  - Firefoxクッキー認証

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

### ヘルプ表示

```bash
.\target\release\downloader.exe --help
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

## バージョン履歴

### v1.1.0 (2026-01-20)
- ✨ **対話的ループモード**: 複数URLを連続してダウンロード可能に
- ✨ **単一URLモード** (`--url`): 1つのURLを指定してダウンロード
- ✨ **バッチモード** (`--urls`): 複数URLを一度にダウンロード
- 🎮 終了コマンド対応: `exit`, `quit`, Ctrl+C, Ctrl+Z/D

### v1.0.0 (2026-01-20)
- 🎉 初回リリース
- ✨ yt-dlp自動ダウンロード機能
- ✨ プラットフォーム自動検出 (YouTube, Twitch, Twitter/X)
- ✨ プラットフォーム別最適化設定

## 元のPython版との違い

- Rustで実装され、型安全性とパフォーマンスが向上
- yt-dlpの自動ダウンロード機能を追加
- 3つの動作モード (対話的/単一/バッチ)
- より詳細なエラーハンドリング
- クロスプラットフォーム対応 (Windows/Linux/macOS)

## ライセンス

BSD-2-Clause
