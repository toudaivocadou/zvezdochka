use crate::site::metadata::RenderableMetadata;
use crate::site::sitemap::SiteMap;
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use eyre::Report;
use maud::{Markup, PreEscaped, html};
use minijinja::Environment;
use pulldown_cmark::html::push_html;
use pulldown_cmark::{Event, Options, Parser};
use seahash::SeaHasher;
use std::hash::Hasher;
use time::Date;
use time::macros::format_description;
use url::Url;

pub fn shorten(content: &str) -> String {
    content.chars().take(150).collect::<String>()
}

pub fn render_markdown(
    rendering_context: &str,
    environment: &Environment,
    meta: &impl RenderableMetadata,
    text: &str,
) -> Result<String, Report> {
    let templated = environment.render_str(text, meta).map_err(|why| {
        Report::msg(format!(
            "In file {rendering_context}, failed to render minijinja template: {why}"
        ))
    })?;

    let options = Options::all();
    let mut output_str_buf = String::new();

    let parser = Parser::new_ext(&templated, options);

    push_html(&mut output_str_buf, parser.map(|event| -> Event { event }));

    Ok(output_str_buf)
}

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

pub fn rewrite_link(site_url: &str, link: String) -> Result<String, Report> {
    if link.starts_with("..") || link.starts_with("#") || link.starts_with("https://") {
        return Ok(link);
    }
    if let Ok(mut url) = Url::parse(&link) {
        url.set_scheme("https")
            .map_err(|_| Report::msg("failed to set URL scheme"))?;
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

pub fn make_path_relative(root: &str, path: impl AsRef<str>) -> String {
    let path = path.as_ref();
    if path.starts_with(root) {
        return path.to_string();
    }
    if path.starts_with("/") {
        return path.to_string();
    }
    return format!("{root}/{path}");
}

pub fn format_date(date: Date) -> String {
    let format = format_description!("[year]-[month]-[day]");
    date.format(format).unwrap()
}

pub fn hash<T: std::hash::Hash>(item: &T) -> u64 {
    let mut seahasher = SeaHasher::default();
    item.hash(&mut seahasher);
    seahasher.finish()
}

pub fn image_or_gray(image_path: Option<&String>) -> &str {
    match image_path {
        Some(p) => p.as_str(),
        None => "gray.jpg",
    }
}

// pub type ItemReference = String;

// pub fn reference(item: impl RenderableMetadata) -> ItemReference {
//     let hash_self = hash(&item);
//     let cachebust = BASE64_URL_SAFE_NO_PAD.encode(hash_self.to_le_bytes());

//     format!("{}-{}", item.title(), cachebust)
// }

pub fn reference<S>(title: &str, known_authors: &[S], unknown_authors: &[S]) -> String
where
    S: AsRef<str>,
{
    let mut hasher = SeaHasher::default();
    hasher.write(title.as_bytes());
    known_authors
        .iter()
        .chain(unknown_authors)
        .for_each(|item| {
            let item = item.as_ref();
            hasher.write(item.as_bytes());
        });
    let cachebust = BASE64_URL_SAFE_NO_PAD.encode(hasher.finish().to_le_bytes());
    format!("{title}-{cachebust}")
}

pub fn known_invalid_link<S>(inner: &S) -> Markup
where
    S: AsRef<str>,
{
    html! {
        a .invalid-link href = "." {
            (inner.as_ref())
        };
    }
}

pub fn author_list<S>(sitemap: &SiteMap, known_authors: &[S], unknown_authors: &[S]) -> Markup
where
    S: AsRef<str>,
{
    if known_authors.is_empty() && unknown_authors.is_empty() {
        return html! {};
    }

    let unknown_authors_rendered = unknown_authors.iter().map(|unknown| {
        html! {
            (known_invalid_link(unknown))
        }
    });

    let known_authors_rendered = known_authors.iter().map(|author| {
        let author = author.as_ref(); // type system magic :D
        // PANIC: This cannot panic because by the time we are here we already checked every meta.
        let member_meta = sitemap.members.get(author).unwrap();
        html! {
            a .member-role .member-bio href = (format!("/members/{}.html", member_meta.name)) {
                (author)
            };
        }
    });

    let mut total_length = known_authors.len() + unknown_authors.len();

    let mut authors_string = String::with_capacity(total_length * 5 + total_length + 2);

    for author_html in known_authors_rendered {
        authors_string.push_str(&author_html.into_string());
        if total_length != 1 {
            authors_string.push_str(", ");
        }
        total_length -= 1;
    }

    for author_html in unknown_authors_rendered {
        authors_string.push_str(&author_html.into_string());
        if total_length != 1 {
            authors_string.push_str(", ");
        }
        total_length -= 1;
    }

    PreEscaped(authors_string)
}
