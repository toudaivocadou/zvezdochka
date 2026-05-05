use maud::{Markup, Render, html};

pub fn html_head(metadata: &impl Render, scripts: &[&str]) -> Markup {
    html! {
        head {
            meta charset="UTF-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            (metadata)
            link rel="stylesheet" href="styles/styles.css";
            link rel="icon" type="image/x-icon" href="/favicon.ico";
            @for script_url in scripts {
                script async defer src=(script_url) {}
            }
        }
    }
}
