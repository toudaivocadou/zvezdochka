use hauchiwa::{
    Tracker,
    loader::{Image, image::ImageFormat},
};
use maud::{Render, html};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::site::{metadata::RenderImageMetadata, templates::partials::navbar::Sections};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemberMeta {
    pub name: String,       // 活動名
    pub ascii_name: String, // 英語のみあり活動名（活動名発音方法） - 注意: これを使ってアイコンのファイルを探します. ケース・センシティブ!!!!!

    pub department: Option<String>, // 学部
    pub position: Option<String>,   // 役職
    pub entry_year: Option<i32>,    // 入年
    pub short: String,              // 自己紹介（短い）

    pub links: Vec<Url>, // SNSリンク
}

impl RenderImageMetadata for &MemberMeta {
    fn render_image_meta(&self, image: Tracker<'_, Image>) -> maud::Markup {
        let path = image
            .get(format!("images/icon/{}.jpg", &self.ascii_name))
            .ok()
            .map(|img| img.get(ImageFormat::WebP))
            .flatten()
            .map(|path| path.as_str());
        match path {
            Some(p) => html! {
                meta property="og:image" content=(p);
            },
            None => html! {},
        }
    }
}

impl Render for &MemberMeta {
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
