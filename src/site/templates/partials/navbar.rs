use std::fmt::Display;

use maud::{Markup, html};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub enum Sections {
    Home,
    Members,
    MemberProfile,
    Activities,
    Join,
    News,
    NewsPost,
    Works,
    WorksPost,
    AlbumPost,
}

impl Sections {
    pub fn opengraph_type(&self) -> &'static str {
        match self {
            Sections::Home => "website",
            Sections::Members => "website",
            Sections::MemberProfile => "profile",
            Sections::Activities => "website",
            Sections::Join => "website",
            Sections::News => "website",
            Sections::NewsPost => "article",
            Sections::Works => "website",
            Sections::WorksPost => "music.song",
            Sections::AlbumPost => "music.album",
        }
    }
}

impl Display for Sections {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Sections::Home => "ホームページ",
            Sections::Members => "メンバーご覧",
            Sections::MemberProfile => "i love amongus sooo much!",
            Sections::Activities => "活動内容",
            Sections::Join => "参加情報",
            Sections::News => "ニュースご覧",
            Sections::NewsPost => "i love the world and everything in it",
            Sections::Works => "メンバー作品ご覧",
            Sections::WorksPost => "join WSOC!",
            Sections::AlbumPost => {
                "john [ano] numba one wynncraf' guild wi' da fines' bri'ish lads around"
            }
        };
        write!(f, "{name}")
    }
}

pub fn navbar(current_section: Sections) -> Markup {
    html! {
        header {
            div .container {
                a href="/index.html" {
                    h1 {
                        "東京大学ボカロP同好会"
                    }
                }
                nav {
                    ul {
                        (navbar_item("/index.html", current_section == Sections::Home, "ホーム"))
                        (navbar_item("/members.html", current_section == Sections::Members || current_section == Sections::MemberProfile, "メンバー紹介"))
                        (navbar_item("/index.html#activities", current_section == Sections::Activities, "活動内容"))
                        (navbar_item("/join.html", current_section == Sections::Join, "入会案内"))
                        (navbar_item("/works.html", current_section == Sections::Works || current_section == Sections::WorksPost, "リリース"))
                        (navbar_item("/news.html", current_section == Sections::News || current_section == Sections::NewsPost, "ニュース"))
                    }
                }
            }
        }
    }
}

fn navbar_item(link: &str, active: bool, inner: &str) -> Markup {
    html! {
        li {
            a .active[active] href=(link) {
                (inner)
            }
        }
    }
}
