# Video Downloader (Rust版)

Pythonの`downloader.py`をRustで書き直した動画ダウンローダーです。yt-dlpを使用して、複数のプラットフォームから動画をダウンロードします。

## 特徴

- 🚀 **自動yt-dlpダウンロード**: yt-dlpがシステムにインストールされていない場合、自動的にGitHub Releasesからダウンロードして`./binaries/`に保存します
- 🎯 **プラットフォーム自動検出**: URL から Twitch、YouTube、Twitter/X を自動検出し、最適な設定でダウンロード
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
# リポジトリをクローン
git clone <repository-url>
cd Downloader

# Releaseビルド
cargo build --release
```

## 使い方

```bash
# 実行
./target/release/downloader

# または、Windowsの場合
.\target\release\downloader.exe
```

起動後、ダウンロードしたい動画のURLを入力してください:

```
=== yt-dlp Video Downloader ===

動画のURLを入力してください: https://www.youtube.com/watch?v=...
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

MIT License

## 元のPython版との違い

- Rustで実装され、型安全性とパフォーマンスが向上
- yt-dlpの自動ダウンロード機能を追加
- より詳細なエラーハンドリング
- クロスプラットフォーム対応 (Windows/Linux/macOS)
