use std::collections::HashMap;
use std::sync::OnceLock;

use crate::SiteData;
use crate::sitemap::SiteMap;
use crate::templates::partials::navbar::Sections;
use camino::Utf8PathBuf;
use hauchiwa::RuntimeError;
use hauchiwa::loader::Content;
use hauchiwa::{Context, Page, loader::Image};
use log::{error, info};
use lol_html::{Settings, element, rewrite_str};
use maud::Markup;
use minijinja::Environment;
use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, html::push_html};
use serde::Serialize;
use url::Url;

pub fn shorten(content: &str) -> String {
    content.chars().take(150).collect::<String>()
}

#[derive(Clone, Debug, PartialEq)]
pub struct SvgData {
    pub path: camino::Utf8PathBuf,
    pub data: Vec<u8>,
}

pub fn render_markdown<C: Serialize>(
    context: &Context<SiteData>,
    environment: &Environment,
    meta: &C,
    text: &str,
) -> Result<String, anyhow::Error> {
    let templated = environment
        .render_str(text, meta)
        .map_err(|why| anyhow::Error::msg(why.to_string()))?;

    let options = Options::all();
    let mut output_str_buf = String::new();

    let parser = Parser::new_ext(&templated, options);

    push_html(
        &mut output_str_buf,
        parser.map(|event| -> Event {
            match event {
                Event::Start(start) => {
                    let tag = match start {
                        Tag::Image {
                            link_type,
                            dest_url,
                            title,
                            id,
                        } => {
                            let url_utf8 = Utf8PathBuf::from(dest_url.as_ref());
                            if let Ok(picture) = image(context, url_utf8) {
                                Tag::Image {
                                    link_type,
                                    dest_url: CowStr::from(picture.to_string()),
                                    title,
                                    id,
                                }
                            } else {
                                Tag::Image {
                                    link_type,
                                    dest_url,
                                    title,
                                    id,
                                }
                            }
                        }
                        other => other,
                    };
                    Event::Start(tag)
                }
                e => e,
            }
        }),
    );

    Ok(output_str_buf)
}

pub fn image(sack: &Context<SiteData>, path: impl AsRef<str>) -> Result<String, RuntimeError> {
    let path = path.as_ref();

    let picture_path = Utf8PathBuf::from(path);
    let image = sack.get::<Image>(&picture_path)?;
    Ok(image.path.to_string())
}

pub struct AudioFile {}

#[allow(dead_code)]
pub fn audio(sack: &Context<SiteData>, path: impl AsRef<str>) -> Result<String, RuntimeError> {
    let path = path.as_ref();

    let audio_path = Utf8PathBuf::from(path);
    let audio = sack.glob_one_with_file::<AudioFile>(audio_path.as_str())?;
    Ok(audio.file.file.to_string())
}

pub fn markup_to_page(
    ctx: &Context<SiteData>,
    path: impl AsRef<str>,
    markup: Markup,
) -> Result<Page, RuntimeError> {
    rewrite_page(ctx, Page::html(path.as_ref(), &markup.0))
}

static SITE_URL: OnceLock<String> = OnceLock::new();

pub fn set_site_url(value: String) {
    SITE_URL.set(value).expect("Failed to set SITE_URL!")
}

// fn lnk(url: impl AsRef<str>) -> String {
//     let root = SITE_URL.get().expect("SITE_URL not set!");

//     slash_guard(root, url.as_ref())
// }

// pub fn site_url() -> String {
//     SITE_URL.get().expect("SITE_URL not set!").to_string()
// }

static EXTERNAL_BINARY_URL: OnceLock<String> = OnceLock::new();

pub fn set_external_bin_url(value: String) {
    EXTERNAL_BINARY_URL
        .set(value)
        .expect("Failed to set EXTERNAL_BINARY_URL!")
}

static SITE_ROOT: OnceLock<String> = OnceLock::new();

pub fn set_site_root(site_root: String) {
    SITE_ROOT.set(site_root).expect("SITE_ROOT not set!")
}

pub fn site_root() -> &'static str {
    SITE_ROOT.get().expect("SITE_ROOT not set!")
}

// fn lnk_s3(url: impl AsRef<str>) -> String {
//     let url = url.as_ref();

//     if let Ok(u) = Url::parse(url) {
//         warn!("Warning: Passed link to lnk that is already a url");
//         return u.to_string();
//     }

//     let root = if url.starts_with("miku:") {
//         EXTERNAL_BINARY_URL
//             .get()
//             .expect("EXTERNAL_BINARY_URL not set!")
//             .to_string()
//     } else {
//         site_url()
//     };

//     slash_guard(&root, url)
// // }

fn slash_guard(root: &str, thing: &str) -> String {
    if root == "." {
        return if thing.starts_with("/") {
            format!("/{}", thing)
        } else {
            thing.to_string()
        };
    }

    if thing.starts_with("/") {
        format!("{root}{}", thing)
    } else {
        format!("{root}/{}", thing)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_metadata_and_final_page<C, RenderFn>(
    context: &Context<SiteData>,
    environment: &Environment,
    sitemap: &SiteMap,
    name_map: &HashMap<String, String>,
    data: &Content<C>,
    page_type: Sections,
    friendly_name: &str,
    final_url: String,
    render_fn: RenderFn,
) -> Result<Page, RuntimeError>
where
    C: Serialize + Send + Sync,
    RenderFn: FnOnce(
        &Context<SiteData>,
        &C,
        &SiteMap,
        &HashMap<String, String>,
        &String,
    ) -> Result<Markup, RuntimeError>,
{
    let meta = &data.meta;
    let content = &data.text;

    let work_rendered = render_markdown(context, environment, &meta, content)
        .map_err(|why| {
            let err = RuntimeError::msg(why.to_string());
            err.context(format!(
                "{} page for {}: render markdown",
                page_type, friendly_name
            ))
        })
        .and_then(|text| {
            render_fn(context, meta, sitemap, name_map, &text).map_err(|why| {
                why.context(format!(
                    "{} page for {}: render final page",
                    page_type, friendly_name
                ))
            })
        });

    if let Err(why) = &work_rendered {
        error!(
            "BUILD-{}: Error while building page {}: {}.",
            context.get_globals().data.build_id,
            final_url,
            why
        );
    }

    Ok(Page::html(final_url, work_rendered?.into_string()))
}

pub fn rewrite_settings(site_url: &str) -> Settings<'_, '_> {
    Settings {
        element_content_handlers: vec![
            // element!("script", |element| {
            //     element.set_attribute("async", "")?;
            //     element.set_attribute("defer", "")?;
            //     Ok(())
            // }),
            // element!("img", |element| {
            //     element.set_attribute("loading", "lazy")?;

            //     Ok(())
            // }),
            // element!("iframe", |element| {
            //     element.set_attribute("loading", "lazy")?;

            //     Ok(())
            // }),
            element!("[href]", |element| {
                let referring_to = match element.get_attribute("href") {
                    Some(r) => r,
                    None => {
                        return Ok(());
                    }
                };

                let rewritten_lnk = rewrite_link(site_url, referring_to)?;
                element.set_attribute("href", &rewritten_lnk)?;

                Ok(())
            }),
            element!("[src]", |element| {
                let referring_to = match element.get_attribute("src") {
                    Some(r) => r,
                    None => {
                        return Ok(());
                    }
                };

                // check if this refer is relative
                let rewritten_lnk = rewrite_link(site_url, referring_to)?;
                element.set_attribute("src", &rewritten_lnk)?;

                Ok(())
            }),
        ],
        ..Settings::default()
    }
}

pub fn rewrite_html(text: &str, settings: Settings<'_, '_>) -> Result<String, RuntimeError> {
    rewrite_str(text, settings).map_err(|why| RuntimeError::msg(why.to_string()))
}

pub fn rewrite_page(context: &Context<SiteData>, mut page: Page) -> Result<Page, RuntimeError> {
    let build_id = context.get_globals().data.build_id;
    let site_url = &context.get_globals().data.site_url;
    let pgpath = &page.path;

    if page.path.starts_with("/") {
        panic!("page path {} starts with illegal /", page.path);
    }
    if let Some(extension) = page.path.extension()
        && extension != "html"
    {
        info!(
            "BUILD-{}: Skipping page {}, not a html page.",
            build_id, pgpath
        );
        return Ok(page);
    }

    info!("BUILD-{}: Rewriting {}", build_id, pgpath);
    let out = rewrite_html(&page.text, rewrite_settings(site_url))?;
    page.text = out;

    Ok(page)
}

pub fn rewrite_link(site_url: &str, link: String) -> Result<String, anyhow::Error> {
    if link.starts_with("..") || link.starts_with("#") || link.starts_with("https://") {
        return Ok(link);
    }
    if let Ok(mut url) = Url::parse(&link) {
        url.set_scheme("https")
            .map_err(|_| anyhow::Error::msg("???"))?;
        return Ok(url.to_string());
    }
    // run our transformations
    if link.starts_with("./") {
        let striped = link.strip_prefix("./").unwrap();
        let fixed = slash_guard(site_url, striped);
        return Ok(fixed);
    }
    if link.starts_with(".") {
        let striped = link.strip_prefix(".").unwrap();
        let fixed = slash_guard(site_url, striped);
        return Ok(fixed);
    }

    Ok(link)
}
