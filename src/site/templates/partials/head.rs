use crate::SiteData;
use crate::metadata::{Metadata, render_metadata};
use camino::Utf8PathBuf;
use hauchiwa::loader::{Script, Style};
use hauchiwa::{Context, ContextError, RuntimeError};
use log::warn;
use maud::{Markup, html};

pub fn html_head(
    sack: &Context<SiteData>,
    metadata: &Metadata,
    scripts: &[&str],
) -> Result<Markup, RuntimeError> {
    let style_path = Utf8PathBuf::from("styles/style.css");
    let style = sack.get::<Style>(&style_path)?.path.as_str();

    let scripts = scripts
        .iter()
        .map(|script| {
            let res = sack
                .get::<Script>(format!("js/{}", script))
                .map(|js| &js.path);
            if let Err(why) = &res {
                warn!(
                    "Failed to find script {} due to {}... Ignoring...",
                    script, why
                )
            }
            res
        })
        .collect::<Vec<Result<&Utf8PathBuf, ContextError>>>();

    Ok(html! {
        head {
            meta charset="UTF-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            (render_metadata(sack, metadata)?)
            link rel="stylesheet" href=(style);
            link rel="icon" type="image/x-icon" href="/favicon.ico";
            @for script_url in scripts {
                @if let Ok(s) = script_url {
                    script src=(s) {}
                }
            }
        }
    })
}
