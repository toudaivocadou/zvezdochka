use crate::site::templates::partials::navbar::Sections;
use maud::{Markup, Render, html};
use serde::Serialize;
use std::hash::Hash;

pub trait RenderableMetadata: Serialize + Render + Hash {
    fn render_image_meta(&self) -> Option<Markup>;

    fn section(&self) -> Sections;

    fn title(&self) -> &str;
}

#[derive(Clone, Debug, Hash, Serialize)]
pub struct GenericMeta {
    pub path: &'static str,
    pub section: Sections,
    pub title: &'static str,
}

impl Render for GenericMeta {
    fn render(&self) -> Markup {
        html! {
            meta property="og:title" content=(self.title);
            meta property="og:site_name" content="東京大学ボカロP同好会 - University of Tokyo Vocaloid Producer Club";
            meta property="og:locale" content="ja_JP";
            meta property="og:type" content=(self.section.opengraph_type());
        }
    }
}

impl RenderableMetadata for GenericMeta {
    fn render_image_meta(&self) -> Option<Markup> {
        None
    }

    fn section(&self) -> Sections {
        self.section
    }

    fn title(&self) -> &str {
        self.title
    }
}

impl<T> RenderableMetadata for Box<T>
where
    T: RenderableMetadata,
{
    fn render_image_meta(&self) -> Option<Markup> {
        self.as_ref().render_image_meta()
    }

    fn section(&self) -> Sections {
        self.as_ref().section()
    }

    fn title(&self) -> &str {
        self.as_ref().title()
    }
}

impl<T> RenderableMetadata for &T
where
    T: RenderableMetadata,
{
    fn render_image_meta(&self) -> Option<Markup> {
        T::render_image_meta(&self)
    }

    fn section(&self) -> Sections {
        T::section(&self)
    }

    fn title(&self) -> &str {
        T::title(&self)
    }
}
