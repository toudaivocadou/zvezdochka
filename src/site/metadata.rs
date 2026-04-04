use hauchiwa::{Many, Tracker, loader::Image};
use maud::{Markup, Render, html};
use time::Date;

use crate::site::templates::partials::navbar::Sections;

pub trait RenderImageMetadata {
    fn render_image_meta(&self, image: Tracker<'_, Image>) -> Markup;
}

// impl<'a> Render for Metadata<'a> {
//     fn render(&self) -> Markup {
//         let og_type = self.section.opengraph_type();

//         html! {
//             meta property="og:title" content=(&self.page_title);
//             // meta property="og:url" content=(canonical_link);
//             meta property="og:site_name" content="東京大学ボカロP同好会 - University of Tokyo Vocaloid Producer Club"; // production -> producer - ありがとーnekojitalter
//             meta property="og:locale" content="ja_JP";
//             meta property="og:type" content=(og_type);
//             @match og_type {
//                 "profile" => {
//                     @if let Some(username) = self.authors.get(0) {
//                         meta property="og:profile:username" content=(username);
//                     }
//                 }
//                 "article" => {
//                     @if let Some(pubtime) = self.date_rendered {
//                         meta property="og:article:published_time" content=(pubtime);
//                     }
//                     @for author in self.authors {
//                         meta property="og:article:author" content=(author);
//                     }
//                     meta property="og:article:section" content=(self.section);
//                 }
//                 _ => ("")
//             }
//             @if let Some(img) = &self.page_image {
//                 meta property="og:image" content=(img);
//             }
//             @if let Some(desc) = &self.description {
//                 meta property="og:description" content=(desc);
//             }
//         }
//     }
// }
