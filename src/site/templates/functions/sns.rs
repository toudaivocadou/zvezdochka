use crate::site::die_linky::SocialLinkType;
use eyre::Report;
use maud::{Markup, html};
use minijinja::Error as JinjaError;
use minijinja::ErrorKind;
use url::Url;

pub fn sns_icon(link: &Url) -> Result<Markup, Report> {
    let link_type = SocialLinkType::from_url(link)?;
    let sns_url_icon = link_type.to_svg_icon();
    let special_style = match link_type {
        // horrible, horrible hack but we roll with it ig
        SocialLinkType::Bluesky => "width: 100%;",
        _ => "",
    };
    Ok(html! {
        a .social-icon .social-icon-size href=(link) {
            img alt=(link) src=(format!("public/social_icons/{}", sns_url_icon)) style=(special_style);
        }
    })
}

pub fn jinja_sns_icon(link: &str) -> Result<String, JinjaError> {
    let url = Url::parse(link)
        .map_err(|why| JinjaError::new(ErrorKind::CannotDeserialize, why.to_string()))?;

    Ok(sns_icon(&url)
        .map_err(|why| JinjaError::new(ErrorKind::InvalidOperation, why.to_string()))?
        .into_string())
}
