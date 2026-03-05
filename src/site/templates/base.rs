use crate::SiteData;
use crate::metadata::Metadata;
use crate::templates::partials::footer::footer;
use crate::templates::partials::head::html_head;
use crate::templates::partials::navbar::navbar;
use hauchiwa::{Context, RuntimeError};
use maud::{DOCTYPE, Markup, Render, html};

pub fn base<'a, Meta>(
    sack: &Context<SiteData>,
    header_metadata: &'a Meta,
    scripts: Option<&[&str]>,
    inner: impl Render,
) -> Result<Markup, RuntimeError>
where
    &'a Meta: Into<&'a Metadata>,
{
    let metadata = Into::into(header_metadata);
    let scripts = match scripts {
        Some(s) => s,
        None => &[],
    };

    Ok(html! {
        (DOCTYPE)
        html lang="ja" {
            (html_head(sack, metadata, scripts)?)
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
