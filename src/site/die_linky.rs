use anyhow::Error;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Hash, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocialLinkType {
    Twitter,
    Xitter,
    Bluesky,
    Youtube,
    NicoDouga,
    Soundcloud,
    Github,
    LinkTree,
    Spotify,
    TikTok,
    Instagram,
    Bandcamp,
    OtherUnknown(String),
}

impl SocialLinkType {
    pub fn to_svg_icon(&self) -> &str {
        match self {
            SocialLinkType::Twitter => "twitter.svg",
            SocialLinkType::Xitter => "twitter.svg", // FIXME
            SocialLinkType::Bluesky => "bluesky.svg",
            SocialLinkType::Youtube => "youtube.svg",
            SocialLinkType::NicoDouga => "niconico.svg",
            SocialLinkType::Soundcloud => "soundcloud.svg",
            SocialLinkType::Github => "github.svg",
            SocialLinkType::LinkTree => "linktree.svg",
            SocialLinkType::Spotify => "spotify.svg",
            SocialLinkType::TikTok => "tiktok.svg",
            SocialLinkType::Instagram => "instagram.svg",
            SocialLinkType::Bandcamp => "bandcamp.svg",
            SocialLinkType::OtherUnknown(_) => "link.svg",
        }
    }

    pub fn from_url(url: &Url) -> Result<SocialLinkType, Error> {
        let domain = url.domain().ok_or(Error::msg(format!("Bad URL: {url}")))?;
        let url_type = match domain {
            "twitter.com" => SocialLinkType::Twitter,
            "x.com" => SocialLinkType::Twitter,
            "bsky.app" => SocialLinkType::Bluesky,
            "youtube.com" | "www.youtube.com" => SocialLinkType::Youtube,
            "soundcloud.com" => SocialLinkType::Soundcloud,
            "nicovideo.jp" | "www.nicovideo.jp" => SocialLinkType::NicoDouga,
            "github.com" => SocialLinkType::Github,
            "linktree.com" | "linktr.ee" => SocialLinkType::LinkTree,
            "spotify.com" => SocialLinkType::Spotify,
            "tiktok.com" => SocialLinkType::TikTok,
            "instagram.com" => SocialLinkType::Instagram,
            other => {
                if other.contains("bandcamp.com") {
                    SocialLinkType::Bandcamp
                } else {
                    SocialLinkType::OtherUnknown(other.to_string())
                }
            }
        };

        Ok(url_type)
    }
}
