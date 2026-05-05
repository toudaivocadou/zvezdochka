use eyre::Report;
use maud::{Markup, html};
use url::Url;

use crate::site::templates::functions::sns::sns_icon;

pub fn footer() -> Result<Markup, Report> {
    Ok(html! {
        footer {
            .container {
                p {
                    "© 2025 東京大学ボカロP同好会"
                }
                .social-links .social-footer {
                    (sns_icon(&Url::parse("https://x.com/toudaivocadou")?)?)
                }
            }
        }
    })
}
