import random
import string
import subprocess
import os

def download_video():
    # ダウンロード先ディレクトリを指定
    download_dir = './'

    # yt-dlp.exe のフルパスを指定
    ytdlp_path = 'yt-dlp'

    # URLを入力させる
    url = input("動画のURLを入力してください: ")

    # URLに基づいてコマンドリストを変更
    if "twitch.tv" in url:
        command = [
            ytdlp_path,
            '-f', '1080p60+bestaudio',  # 最大画質を指定
            '--merge-output-format', 'mp4',
            '--embed-thumbnail',  # サムネイルを埋め込む
            '--add-metadata',  # メタデータを追加
            '--output', os.path.join(download_dir, '%(title)s.%(ext)s'),  # 保存先
            url
        ]
    elif "youtube.com" in url or "youtu.be" in url:
        command = [
            ytdlp_path,
            '--cookies-from-browser', 'firefox',  # FirefoxのCookieを使用
            '-4',  # IPv4を強制
            '-f', 'bestvideo+bestaudio',
            '--merge-output-format', 'mp4',
            '--embed-thumbnail',  # サムネイルを埋め込む
            '--add-metadata',  # メタデータを追加
            '--geo-bypass-country', 'JP',  # 日本からのアクセスを強制
            '--output', os.path.join(download_dir, '%(title)s.%(ext)s'),  # 保存先
            url
        ]
    elif "twitter.com" in url or "x.com" in url:
        command = [
            ytdlp_path,
            '--merge-output-format', 'mp4',
            '--embed-thumbnail',  # サムネイルを埋め込む
            '--add-metadata',  # メタデータを追加
            '--output', os.path.join(download_dir, '%(title)s.%(ext)s'),  # 保存先
            url
        ]
    else:
        print("最高画質での保存ができない場合があります。")
        command = [
            ytdlp_path,
            '--merge-output-format', 'mp4',
            '--output', os.path.join(download_dir, f'%(title)s.%(ext)s'),
            '--embed-thumbnail',
            '--add-metadata',
            '--geo-bypass-country', 'JP',
            '-f', 'bestvideo+bestaudio/best',
            '--no-playlist',
            '--cookies-from-browser', 'firefox',
            '--user-agent', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.74 Safari/537.36',
            '--write-sub',
            '--sub-lang', 'all',  # すべての利用可能な字幕をダウンロード
            '--sub-format', 'best',  # 最適な字幕フォーマット
            '--convert-subs', 'srt',  # 字幕をSRT形式に変換
            '--ignore-errors',  # エラーが発生しても続行
            url
        ]

    # コマンドの実行
    try:
        # 出力を無視してコマンドを実行
        subprocess.run(command, check=True)
        print("ダウンロードが完了しました。")
    except subprocess.CalledProcessError as e:
        # エラーが発生した場合の処理
        print(f"エラーが発生しました: {e}")

if __name__ == "__main__":
    download_video()
