use anyhow::Error;
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use hauchiwa::{
    Tracker,
    loader::{Image, Script, Stylesheet},
};
use lol_html::{Settings, element, rewrite_str};
use std::borrow::Cow;

pub struct TrackerSet<'a> {
    pub images: Tracker<'a, Image>,
    pub scripts: Tracker<'a, Script>,
    pub styles: Tracker<'a, Stylesheet>,
}

pub fn fixup_html(
    build_id: Option<u64>,
    trackers: TrackerSet,
    html: String,
) -> Result<String, Error> {
    let settings = Settings {
        element_content_handlers: vec![
            element!("img[src]", |el| {
                let current_src = el.get_attribute("src").unwrap(); // PANIC: Can't panic, since we selected for img[src]
                if current_src.starts_with("http://") || current_src.starts_with("https://") {
                    return Ok(());
                }
                let new_src = fixup_image(&trackers.images, Cow::Borrowed(current_src.as_str()))
                    .map_err(|why| Into::<Error>::into(why))?;
                el.set_attribute("src", fixup_abs_link(build_id, new_src).as_ref());
                Ok(())
            }),
            element!("script[src]", |el| {
                let current_src = el.get_attribute("src").unwrap();
                if current_src.starts_with("http://") || current_src.starts_with("https://") {
                    return Ok(());
                }
                let new_src = fixup_scripts(&trackers.scripts, Cow::Borrowed(current_src.as_str()))
                    .map_err(|why| Into::<Error>::into(why))?;
                el.set_attribute("src", fixup_abs_link(build_id, new_src).as_ref());
                Ok(())
            }),
            element!("link[href]", |el| {
                let current_href = el.get_attribute("href").unwrap();
                if current_href.starts_with("http://") || current_href.starts_with("https://") {
                    return Ok(());
                }

                let rel_type = match el.get_attribute("rel") {
                    Some(r) => r,
                    None => return Ok(()),
                };

                let cow_href = Cow::Borrowed(current_href.as_str());

                let new_href = match rel_type.as_str() {
                    "stylesheet" => fixup_styles(&trackers.styles, cow_href)
                        .map_err(|why| Into::<Error>::into(why))?,
                    "script" => fixup_scripts(&trackers.scripts, cow_href)
                        .map_err(|why| Into::<Error>::into(why))?,
                    _ => {
                        return Ok(());
                    }
                };

                el.set_attribute("href", fixup_abs_link(build_id, new_href).as_ref());
                Ok(())
            }),
            element!("meta[property^=\"og:\"]", |el| {
                let property = el.get_attribute("property").unwrap();
                let current_content = match el.get_attribute("content") {
                    Some(c) => c,
                    None => return Ok(()),
                };

                let cow_current_content = Cow::Borrowed(current_content.as_str());

                let new_content = match property.as_str() {
                    "og:url" => cow_current_content,
                    "og:image" => fixup_image(&trackers.images, cow_current_content)
                        .map_err(|why| Into::<Error>::into(why))?,
                    _ => {
                        return Ok(());
                    }
                };

                el.set_attribute("content", fixup_abs_link(build_id, new_content).as_ref());
                Ok(())
            }),
            element!("a[href]", |el| {
                let current_href = el.get_attribute("href").unwrap();
                let current_href: Cow<'_, str> = Cow::Owned(current_href);

                let new_href = if current_href.starts_with("https://") {
                    current_href
                } else if current_href.starts_with("http://") {
                    rewrite_external_http_links(current_href)
                } else {
                    fixup_abs_link(build_id, current_href)
                };
                el.set_attribute("href", new_href.as_ref());
                Ok(())
            }),
        ],
        ..Settings::default()
    };

    rewrite_str(&html, settings).map_err(|why| Error::new(why))
}

fn build_id_to_str(build_id: u64) -> String {
    BASE64_URL_SAFE_NO_PAD.encode(format!("{build_id}"))
}

fn fixup_abs_link<'a>(build_id: Option<u64>, destination: Cow<'a, str>) -> Cow<'a, str> {
    match build_id {
        Some(build) => {
            if destination.starts_with("/") {
                Cow::Owned(format!("/{}{}", build_id_to_str(build), destination))
            } else {
                Cow::Owned(format!("/{}/{}", build_id_to_str(build), destination))
            }
        }
        None => destination,
    }
}

fn fixup_image<'a>(
    images: &Tracker<Image>,
    destination: Cow<'a, str>,
) -> Result<Cow<'a, str>, Error> {
    if !(destination.ends_with(".jpg")
        || destination.ends_with(".jpeg")
        || destination.ends_with(".png")
        || destination.ends_with(".webp"))
    {
        return Ok(destination);
    }

    let intermediary = prefixing_it_up("images", destination);

    images
        .get(intermediary)
        .map(|img| Cow::Owned(img.default.to_string()))
        .map_err(|why| Error::new(why))
}

fn fixup_styles<'a>(
    styles: &Tracker<Stylesheet>,
    destination: Cow<'a, str>,
) -> Result<Cow<'a, str>, Error> {
    if !destination.ends_with(".css") {
        return Ok(destination);
    }

    let intermediary = prefixing_it_up("styles", destination);

    styles
        .get(intermediary)
        .map(|stylesheet| Cow::Owned(stylesheet.path.to_string()))
        .map_err(|why| Error::new(why))
}

fn fixup_scripts<'a>(
    scripts: &Tracker<Script>,
    destination: Cow<'a, str>,
) -> Result<Cow<'a, str>, Error> {
    if !destination.ends_with(".js") {
        return Ok(destination);
    }

    let intermediary = prefixing_it_up("scripts", destination);

    scripts
        .get(intermediary)
        .map(|script| Cow::Owned(script.path.to_string()))
        .map_err(|why| Error::new(why))
}

fn prefixing_it_up<'a>(prefix: &'static str, destination: Cow<'a, str>) -> Cow<'a, str> {
    if destination.starts_with(prefix) {
        destination
    } else if destination.starts_with(&format!("/{prefix}")) {
        Cow::Owned(destination.strip_prefix("/").unwrap().to_string())
    } else {
        Cow::Owned(format!("{prefix}/{}", destination))
    }
}

fn rewrite_external_http_links<'a>(destination: Cow<'a, str>) -> Cow<'a, str> {
    if destination.starts_with("http") {
        let temp = destination.strip_prefix("http").unwrap();
        return Cow::Owned(format!("https{temp}"));
    }
    destination
}
