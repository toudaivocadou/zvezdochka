use crate::site::{
    metadata::RenderableMetadata,
    templates::partials::{footer::footer, head::html_head, navbar::navbar},
};
use camino::Utf8PathBuf;
use eyre::Report;
use maud::{DOCTYPE, Markup, Render, html};

pub fn base(
    metadata: &impl RenderableMetadata,
    inner: impl Render,
    scripts: &[&Utf8PathBuf],
    style: &[&Utf8PathBuf],
) -> Result<Markup, Report> {
    Ok(html! {
        (DOCTYPE)
        html lang="ja" {
            (html_head(metadata, scripts, style))
            body {
                (navbar(metadata.section()))
                .main-content-container {
                    (inner)
                }
                (footer()?)
            }
        }
    })
}
