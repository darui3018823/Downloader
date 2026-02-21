#[derive(Debug)]
pub enum Platform {
    Twitch,
    YouTube,
    Twitter,
    Niconico,
    SoundCloud,
    Instagram,
    TikTok,
    Bilibili,
    Generic,
}

impl Platform {
    pub fn detect(url: &str) -> Self {
        let lower = url.to_ascii_lowercase();

        if lower.contains("twitch.tv") {
            Platform::Twitch
        } else if lower.contains("youtube.com") || lower.contains("youtu.be") {
            Platform::YouTube
        } else if lower.contains("twitter.com") || lower.contains("x.com") {
            Platform::Twitter
        } else if lower.contains("nicovideo.jp") || lower.contains("nico.ms") {
            Platform::Niconico
        } else if lower.contains("soundcloud.com") {
            Platform::SoundCloud
        } else if lower.contains("instagram.com") {
            Platform::Instagram
        } else if lower.contains("tiktok.com") {
            Platform::TikTok
        } else if lower.contains("bilibili.com") || lower.contains("b23.tv") {
            Platform::Bilibili
        } else {
            Platform::Generic
        }
    }
}
