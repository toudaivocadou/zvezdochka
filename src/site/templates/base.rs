use eyre::Report;
use maud::{DOCTYPE, Markup, Render, html};

use crate::site::templates::partials::head::html_head;

pub fn base(
    metadata: &impl Render,
    inner: impl Render,
    scripts: Option<&[&str]>,
) -> Result<Markup, Report> {
    let scripts = match scripts {
        Some(s) => s,
        None => &[],
    };

    Ok(html! {
        (DOCTYPE)
        html lang="ja" {
            (html_head(metadata, scripts)?)
            body {
                (navbar(metadata.section))
                .main-content-container {
                    (inner)
                }
                (footer(sack)?)
            }
        }
    })
}
