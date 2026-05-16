use maud::{Markup, Render, html};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::site::{metadata::RenderableMetadata, templates::partials::navbar::Sections};

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkTitleOrSource {
    Source(Url),
    Title(String),
}

#[derive(Clone, Debug, Hash, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct MemberMeta {
    pub name: String,       // 活動名
    pub ascii_name: String, // 英語のみあり活動名（活動名発音方法） - 注意: これを使ってアイコンのファイルを探します. ケース・センシティブ!!!!!

    #[serde(default)]
    pub department: Option<String>, // 学部
    #[serde(default)]
    pub position: Option<String>, // 役職
    #[serde(default)]
    pub entry_year: Option<i32>, // 入年
    pub short: String, // 自己紹介（短い）

    #[serde(default)]
    pub links: Vec<Url>, // SNSリンク

                         // #[serde(default)]
                         // pub featured_works: Vec<WorkTitleOrSource>,
}

impl RenderableMetadata for MemberMeta {
    fn render_image_meta(&self) -> Option<Markup> {
        Some(html! {
            meta property="og:image" content=(format!("icon/{}.jpg", &self.ascii_name));
        })
    }

    fn section(&self) -> Sections {
        Sections::MemberProfile
    }

    fn title(&self) -> &str {
        &self.name
    }
}

impl Render for MemberMeta {
    fn render(&self) -> maud::Markup {
        let og_type = Sections::MemberProfile.opengraph_type();

        html! {
            meta property="og:title" content=(&self.name);
            meta property="og:site_name" content="東京大学ボカロP同好会 - University of Tokyo Vocaloid Producer Club";
            meta property="og:locale" content="ja_JP";
            meta property="og:type" content=(og_type);
            meta property="og:profile:username" content=(&self.ascii_name);
            meta property="og:description" content=(&self.short);
        }
    }
}
