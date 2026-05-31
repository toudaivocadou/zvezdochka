use crate::site::album::AlbumMeta;
use crate::site::fixup::{TrackerSet, fixup_html};
use crate::site::member::MemberMeta;
use crate::site::metadata::GenericMeta;
use crate::site::news::NewsMeta;
use crate::site::sitemap::{MemberRef, SiteMap};
use crate::site::templates::base::base;
use crate::site::templates::functions::embed::jinja_embed;
use crate::site::templates::functions::member::jinja_member;
use crate::site::templates::functions::sns::jinja_sns_icon;
use crate::site::templates::index::index;
use crate::site::templates::join::join_vocadou;
use crate::site::templates::members::{member_detail, member_index};
use crate::site::templates::news::{NEWS_MISSING_AUTHOR, news_detail};
use crate::site::templates::partials::navbar::Sections;
use crate::site::templates::works::{album_detail, work_detail};
use crate::site::util::{BuildSteps, MajorContext, SubBuildStep, reference, render_markdown};
use crate::site::work::WorkMeta;
use anyhow::Error;
use clap::{Parser, ValueEnum};
use hauchiwa::error::HauchiwaError;
use hauchiwa::tracing::{error, info, warn};
use hauchiwa::{Blueprint, Output};
use indexmap::IndexMap;
use minijinja::Environment;
use minijinja_contrib::add_to_environment;
use minijinja_contrib::pycompat::unknown_method_callback;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use url::Url;

mod album;
mod die_linky;
mod fixup;
mod member;
mod metadata;
mod news;
mod sitemap;
pub mod templates;
mod util;
mod work;

pub const FRONT_MATTER_SPLIT: &str = "===";

#[derive(Clone, Debug, PartialEq)]
pub struct BuildData {
    pub name_map: HashMap<String, String>,
}

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "0")]
    build_id: u64,
    #[clap(short, long, default_value = ".")]
    data_root: PathBuf,
    #[clap(short, long, default_value = "https://miku.toudaivocadou.org")]
    external_url_root: Url,
    #[clap(short, long, default_value = "https://toudaivocadou.org")]
    site_url: String,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
enum Mode {
    Build,
    Watch,
}

#[derive(Clone, Debug)]
pub struct SiteData {
    pub site_url: String,
    pub make_vendoring: bool,
    pub offline_mode: bool,
    pub build_id: Option<u64>,
}

pub fn buildsite(
    build_id: Option<u64>,
    site_url: String,
    source_path: String,
    make_vendoring: bool,
    offline_mode: bool,
) -> Result<(), HauchiwaError> {
    let start_time = Instant::now();
    let site_data = SiteData {
        site_url,
        make_vendoring,
        offline_mode,
        build_id,
    };
    info!("Starting Site Build. サイト建築始め中");
    info!("Base Site URL: {}", &site_data.site_url);
    info!("Site Source Path: {}", &source_path);
    // TODO: Vendoring.
    if make_vendoring {
        warn!(
            "Will make request vendors and place them into {}/.vendor",
            &source_path
        );
    }
    if offline_mode {
        warn!("Running in offline mode. Any non-vendored items will return an error.")
    }

    let mut config = Blueprint::<SiteData>::new();

    // define base image, script, js resources

    let styles = config
        .load_css()
        .entry(format!("{source_path}/styles/*.css"))?
        .register();

    let images = config
        .load_images()
        .glob(format!("{source_path}/images/**/*.png"))?
        .glob(format!("{source_path}/images/**/*.jpg"))?
        .glob(format!("{source_path}/images/**/*.jpeg"))?
        .glob(format!("{source_path}/images/**/*.avif"))?
        .glob(format!("{source_path}/images/**/*.gif"))?
        .register();

    let scripts = config
        .load_esbuild()
        .entry(format!("{source_path}/js/*.js"))?
        .bundle(true)
        .minify(true)
        .register();

    // build minijinja environment

    let environment = config.task().name("Build Minijinja Environment").run(|_| {
        let mut environment = Environment::new();
        environment.add_function("sns_embed", jinja_embed);
        environment.add_function("member", jinja_member);
        environment.add_function("sns_icon", jinja_sns_icon);
        add_to_environment(&mut environment);
        environment.set_unknown_method_callback(unknown_method_callback);
        Ok(environment)
    });

    // load site posts

    let members = config
        .load_documents::<MemberMeta>()
        .glob(format!("{source_path}/works/[!_]*.md"))?
        .register();

    let works = config
        .load_documents::<WorkMeta>()
        .glob(format!("{source_path}/works/[!_]*.md"))?
        .register();

    let albums = config
        .load_documents::<AlbumMeta>()
        .glob(format!("{source_path}/works/[!_]*.md"))?
        .register();

    let news = config
        .load_documents::<NewsMeta>()
        .glob(format!("{source_path}/works/[!_]*.md"))?
        .register();

    // build SiteMap

    let sitemap = config.task().using((members, works, albums, news)).merge(
        |site_data, (mems, works, albs, newses)| {
            let mut members = mems
                .values()
                .map(|m| (m.matter.ascii_name.clone(), *m.matter.clone()))
                .collect::<IndexMap<MemberRef, MemberMeta>>();

            members.sort_by(|_, a, _, b| {
            let a_str = a.position.clone().unwrap_or_default();
            let b_str = b.position.clone().unwrap_or_default();

            if a_str == "代表" {
                return Ordering::Less;
            } else if b_str == "代表" {
                return Ordering::Greater;
            }

            if a_str == "副代表" {
                return Ordering::Less;
            } else if b_str == "副代表" {
                return Ordering::Greater;
            }

            if a_str == "広報" {
                return Ordering::Less;
            } else if b_str == "広報" {
                return Ordering::Greater;
            }

            if a.position.is_some() && b.position.is_none() {
                return Ordering::Less;
            } else if a.position.is_none() && b.position.is_some() {
                return Ordering::Greater;
            } else if a.position == b.position {
                return a.name.cmp(&b.name);
            }

            a.name.cmp(&b.name)
        });

            let mut works = works
                .values()
                .map(|w| *w.matter)
                .collect::<Vec<WorkMeta>>();

            works.sort_by(|a, b| {
                a.date.cmp(&b.date)
            });
            works.reverse();

            let mut albums = albs
                .values()
                .map(|a| *a.matter)
                .collect::<Vec<AlbumMeta>>();

            albums.sort_by(|a, b| {
                a.date.cmp(&b.date)
            });
            albums.reverse();

            let mut news = newses
                .values()
                .map(|n| *n.matter)
                .collect::<Vec<NewsMeta>>();

            news.sort_by(|a, b| {
                a.date.cmp(&b.date)
            });
            news.reverse();

            let mut should_error = false;

            // ensure sitemap is good
            for work in &works {
                for author in &work.authors {
                    if !members.contains_key(author) {
                        should_error = true;
                        error!("Sitemap: 作品 {}で投稿者{}を見つけませんでした。サイトで登録していない投稿者は additional_authors 欄に入力してください。", work.title, author)
                    }
                }
            }

            for album in &albums {
                for author in &album.authors {
                    if !members.contains_key(author) {
                        should_error = true;
                        error!("Sitemap: アルバム {}で投稿者{}を見つけませんでした。サイトで登録していない投稿者は additional_authors 欄に入力してください。", album.title, author)
                    }
                }

                for song in &album.tracks {
                    if song.external {
                        continue;
                    }

                    for song_author in &song.authors {
                        if !members.contains_key(song_author) {
                            should_error = true;
                            error!("Sitemap: アルバム {}の曲{}投稿者{}を見つけませんでした。サイトで登録していない投稿者は additional_authors 欄に入力してください。", album.title, song.title, song_author)
                        }
                    }

                    for illust in &album.illustrations {
                        for illustrator in &illust.illustrators {
                            if !members.contains_key(illustrator) {
                                should_error = true;
                                error!("Sitemap: アルバム {}のイラスト{}投稿者{}を見つけませんでした。サイトで登録していない投稿者は additional_authors 欄に入力してください。", album.title, illust.image, illustrator)
                            }
                        }
                    }
                }
            }

            for post in &news {
                if let Some(author) = &post.author {
                    if !members.contains_key(author) {
                        should_error = true;
                        error!("Sitemap: ニュース {}で投稿者{}を見つけませんでした。サイトで登録していない投稿者は additional_authors 欄に入力してください。", post.title, author)
                    }
                }
            }

            // ensure member works are valid
            // for member in members.values() {
            //     for featured_work in &member.featured_works {
            //         match featured_work {
            //             WorkTitleOrSource::Source(url) => {
            //                 let found = works.iter().find(|meta| &meta.source == url).is_some();
            //                 if !found {
            //                     should_error = true;
            //                     error!("Sitemap: Member: Featured Work: {}のメンバーページで注目作品(URL {})を見つけませんでした。", member.ascii_name, url);
            //                 }
            //             },
            //             WorkTitleOrSource::Title(title) => {
            //                 let found = works.iter().find(|meta| &meta.title == title).is_some();
            //                 if !found {
            //                     should_error = true;
            //                     error!("Sitemap: Member: Featured Work: {}のメンバーページで注目作品(作名 {})を見つけませんでした。", member.ascii_name, title);
            //                 }
            //             },
            //         }
            //     }
            // }
            if should_error {
                return Err(Error::msg("Errors occured during sitemap construction."))
            }


            let site_map = SiteMap {
                members,
                news,
                works,
                albums,
            };
            Ok(site_map)
        },
    );

    let _work_pages = config
        .task()
        .each(works)
        .using((environment, sitemap, images, scripts, styles))
        .map(
            |site_data, work, (environment, sitemap, image, scripts, styles)| {
                let major_context = MajorContext {
                    step: BuildSteps::Works,
                    file: Some(work.meta.path.clone()),
                    build_id: site_data.env.data.build_id,
                };

                let rendered_markdown = render_markdown(&environment, &work.matter, &work.text)
                    .map_err(|why| {
                        why.context(major_context.with_substep(SubBuildStep::ParsingMarkdown))
                    })?;
                let templated_html = work_detail(sitemap, &work.matter, rendered_markdown)
                    .map_err(|why| {
                        why.context(major_context.with_substep(SubBuildStep::Templating))
                    })?;
                let full_html = base(&work.matter, templated_html, &[], &[]).map_err(|why| {
                    why.context(major_context.with_substep(SubBuildStep::BaseHTMLFilling))
                })?;

                let path = reference(
                    &work.matter.title,
                    &work.matter.authors,
                    &work.matter.additional_authors,
                );

                let trackers = TrackerSet {
                    images: image,
                    scripts,
                    styles,
                };

                let html_fixup = fixup_html(
                    site_data.env.data.build_id,
                    trackers,
                    full_html.into_string(),
                )
                .map_err(|why| why.context(major_context.with_substep(SubBuildStep::Fixup)))?;

                Ok(Output::html(
                    format!("/works/releases/{path}.html"),
                    html_fixup,
                ))
            },
        );

    let _album_pages = config
        .task()
        .each(albums)
        .using((environment, sitemap, images, scripts, styles))
        .map(
            |site_data, album, (environment, sitemap, image, scripts, styles)| {
                let major_context = MajorContext {
                    step: BuildSteps::Albums,
                    file: Some(album.meta.path.clone()),
                    build_id: site_data.env.data.build_id,
                };

                let rendered_markdown = render_markdown(&environment, &album.matter, &album.text)
                    .map_err(|why| {
                    why.context(major_context.with_substep(SubBuildStep::ParsingMarkdown))
                })?;
                let templated_html = album_detail(sitemap, &album.matter, rendered_markdown)
                    .map_err(|why| {
                        why.context(major_context.with_substep(SubBuildStep::Templating))
                    })?;
                let full_html = base(&album.matter, templated_html, &[], &[]).map_err(|why| {
                    why.context(major_context.with_substep(SubBuildStep::BaseHTMLFilling))
                })?;

                let path = reference(
                    &album.matter.title,
                    &album.matter.authors,
                    &album.matter.additional_authors,
                );

                let trackers = TrackerSet {
                    images: image,
                    scripts,
                    styles,
                };

                let html_fixup = fixup_html(
                    site_data.env.data.build_id,
                    trackers,
                    full_html.into_string(),
                )
                .map_err(|why| why.context(major_context.with_substep(SubBuildStep::Fixup)))?;

                Ok(Output::html(
                    format!("/albums/releases/{path}/index.html"),
                    html_fixup,
                ))
            },
        );

    let _news_pages = config
        .task()
        .each(news)
        .using((environment, sitemap, images, scripts, styles))
        .map(
            |site_data, news, (environment, sitemap, image, scripts, styles)| {
                let major_context = MajorContext {
                    step: BuildSteps::News,
                    file: Some(news.meta.path.clone()),
                    build_id: site_data.env.data.build_id,
                };

                let rendered_markdown = render_markdown(&environment, &news.matter, &news.text)
                    .map_err(|why| {
                        why.context(major_context.with_substep(SubBuildStep::ParsingMarkdown))
                    })?;
                let templated_html = news_detail(sitemap, &news.matter, rendered_markdown)
                    .map_err(|why| {
                        why.context(major_context.with_substep(SubBuildStep::Templating))
                    })?;
                let full_html = base(&news.matter, templated_html, &[], &[]).map_err(|why| {
                    why.context(major_context.with_substep(SubBuildStep::BaseHTMLFilling))
                })?;

                let path = reference(
                    &news.matter.title,
                    &[&news
                        .matter
                        .author
                        .as_ref()
                        .map(|x| x.as_str())
                        .unwrap_or(NEWS_MISSING_AUTHOR)],
                    &[],
                );

                let trackers = TrackerSet {
                    images: image,
                    scripts,
                    styles,
                };

                let html_fixup = fixup_html(
                    site_data.env.data.build_id,
                    trackers,
                    full_html.into_string(),
                )
                .map_err(|why| why.context(major_context.with_substep(SubBuildStep::Fixup)))?;

                Ok(Output::html(format!("/news/{path}/index.html"), html_fixup))
            },
        );

    let _member_pages = config
        .task()
        .each(members)
        .using((environment, sitemap, images, scripts, styles))
        .map(
            |site_data, members, (environment, sitemap, image, scripts, styles)| {
                let major_context = MajorContext {
                    step: BuildSteps::Members,
                    file: Some(members.meta.path.clone()),
                    build_id: site_data.env.data.build_id,
                };
                let rendered_markdown =
                    render_markdown(&environment, &members.matter, &members.text).map_err(
                        |why| {
                            why.context(major_context.with_substep(SubBuildStep::ParsingMarkdown))
                        },
                    )?;
                let templated_html = member_detail(sitemap, &members.matter, rendered_markdown)
                    .map_err(|why| {
                        why.context(major_context.with_substep(SubBuildStep::Templating))
                    })?;
                let full_html = base(&members.matter, templated_html, &[], &[]).map_err(|why| {
                    why.context(major_context.with_substep(SubBuildStep::BaseHTMLFilling))
                })?;

                let trackers = TrackerSet {
                    images: image,
                    scripts,
                    styles,
                };

                let html_fixup = fixup_html(
                    site_data.env.data.build_id,
                    trackers,
                    full_html.into_string(),
                )
                .map_err(|why| why.context(major_context.with_substep(SubBuildStep::Fixup)))?;

                Ok(Output::html(
                    format!("/members/{}/index.html", &members.matter.ascii_name),
                    html_fixup,
                ))
            },
        );

    let _member_index_page = config
        .task()
        .name("Member Index Page")
        .using((sitemap, images, scripts, styles))
        .merge(|site_data, (sitemap, images, scripts, styles)| {
            let major_context = MajorContext {
                step: BuildSteps::MemberIndex,
                file: None,
                build_id: site_data.env.data.build_id,
            };
            let member_index = member_index(sitemap).map_err(|why| {
                why.context(major_context.with_substep(SubBuildStep::BaseHTMLFilling))
            })?;

            let member_index_metadata = GenericMeta {
                path: "/members/index.html",
                section: Sections::Members,
                title: "メンバー紹介",
            };

            let full_html = base(&member_index_metadata, member_index, &[], &[])?;
            let trackers = TrackerSet {
                images,
                scripts,
                styles,
            };

            let html_fixup = fixup_html(
                site_data.env.data.build_id,
                trackers,
                full_html.into_string(),
            )
            .map_err(|why| why.context(major_context.with_substep(SubBuildStep::Fixup)))?;

            Ok(Output::html("/members/index.html", html_fixup))
        });

    // begin static construction

    let _join_page = config
        .task()
        .name("Join Page")
        .using((images, scripts, styles))
        .merge(|site_data, (images, scripts, styles)| {
            let major_context = MajorContext {
                step: BuildSteps::JoinPage,
                file: None,
                build_id: site_data.env.data.build_id,
            };
            let page = join_vocadou();

            let metadata = GenericMeta {
                path: "/join.html",
                section: Sections::Join,
                title: "参加案内",
            };

            let full_html = base(&metadata, page, &[], &[])?;
            let trackers = TrackerSet {
                images,
                scripts,
                styles,
            };

            let html_fixup = fixup_html(
                site_data.env.data.build_id,
                trackers,
                full_html.into_string(),
            )
            .map_err(|why| why.context(major_context.with_substep(SubBuildStep::Fixup)))?;

            Ok(Output::html(metadata.path, html_fixup))
        });

    let _join_page = config
        .task()
        .name("Join Page")
        .using((images, scripts, styles))
        .merge(|site_data, (images, scripts, styles)| {
            let major_context = MajorContext {
                step: BuildSteps::JoinPage,
                file: None,
                build_id: site_data.env.data.build_id,
            };
            let page = join_vocadou();

            let metadata = GenericMeta {
                path: "/join.html",
                section: Sections::Join,
                title: "参加案内",
            };

            let full_html = base(&metadata, page, &[], &[])?;
            let trackers = TrackerSet {
                images,
                scripts,
                styles,
            };

            let html_fixup = fixup_html(
                site_data.env.data.build_id,
                trackers,
                full_html.into_string(),
            )
            .map_err(|why| why.context(major_context.with_substep(SubBuildStep::Fixup)))?;

            Ok(Output::html(metadata.path, html_fixup))
        });

    let _index_page = config
        .task()
        .name("Index Page")
        .using((images, scripts, styles))
        .merge(|site_data, (images, scripts, styles)| {
            let major_context = MajorContext {
                step: BuildSteps::IndexPage,
                file: None,
                build_id: site_data.env.data.build_id,
            };
            let page = index();

            let metadata = GenericMeta {
                path: "/index.html",
                section: Sections::Home,
                title: "ホーム",
            };

            let full_html = base(&metadata, page, &[], &[])?;
            let trackers = TrackerSet {
                images,
                scripts,
                styles,
            };

            let html_fixup = fixup_html(
                site_data.env.data.build_id,
                trackers,
                full_html.into_string(),
            )
            .map_err(|why| why.context(major_context.with_substep(SubBuildStep::Fixup)))?;

            Ok(Output::html(metadata.path, html_fixup))
        });

    let mut website = config
        .copy_static(format!("{source_path}/public"), "")
        .finish();

    let _diagnostics = website.build(site_data)?;
    let end_time = Instant::now();
    let build_time = end_time.duration_since(start_time);
    info!(
        "Site build {:?} complete. Took {} seconds.",
        build_id,
        build_time.as_secs_f32()
    );
    Ok(())
}
