use camino::Utf8PathBuf;
use maud::{Markup, Render, html};

pub fn html_head(
    metadata: &impl Render,
    scripts: &[&Utf8PathBuf],
    styles: &[&Utf8PathBuf],
) -> Markup {
    html! {
        head {
            meta charset="UTF-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            (metadata)
            @for script in scripts {
                script async defer src=(script);
            }
            @for style in styles {
                link rel="stylesheet" href=(style);
            }
            link rel="icon" type="image/x-icon" href="/favicon.ico";
            @for script_url in scripts {
                script async defer src=(script_url) {}
            }
        }
    }
}
