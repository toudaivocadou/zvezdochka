use crate::site::album::AlbumMeta;
use crate::site::member::MemberMeta;
use crate::site::news::NewsMeta;
use crate::site::read::{
    parse_front_matter_and_fetch_contents, parse_post_meta, parse_work_meta, robots_txt,
};
use crate::site::sitemap::SiteMap;
use crate::site::templates::error::notfound;
use crate::site::templates::functions::embed::{embed, jinja_embed};
use crate::site::templates::functions::member::jinja_member;
use crate::site::templates::index::index;
use crate::site::templates::join::join_vocadou;
use crate::site::templates::members::{member_detail, members as member_overview};
use crate::site::templates::news::{news_posts, post_detail, post_reference};
use crate::site::templates::partials::navbar::Sections;
use crate::site::templates::works::{
    album_detail, album_reference, work_detail, work_reference, works as works_overview,
};
use crate::site::util::{
    AudioFile, SvgData, markup_to_page, render_metadata_and_final_page, rewrite_html, rewrite_link,
    rewrite_page, rewrite_settings, set_external_bin_url, set_site_root, set_site_url, site_root,
};
use crate::site::work::{DisplayWorkMeta, WorkMeta};
use clap::{Parser, ValueEnum};
use hauchiwa::error::HauchiwaError;
use hauchiwa::loader::Content;
use hauchiwa::{Blueprint, RuntimeError};
use hauchiwa::{Page, Website, loader};
use log::{error, info};
use maud::{Render, html};
use minijinja::Environment;
use minijinja_contrib::add_to_environment;
use minijinja_contrib::pycompat::unknown_method_callback;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use url::Url;

mod album;
mod die_linky;
mod member;
mod metadata;
mod news;
mod optimize;
mod read;
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
}

pub fn buildsite(site_url: String, source_path: String) -> Result<(), HauchiwaError> {
    let time_start = Instant::now();
    let site_data = SiteData { site_url };
    info!("Starting Site Build. サイト建築始め中");
    info!("Base Site URL: {site_url}");
    info!("Site Source Path: {source_path}");

    let mut config = Blueprint::<SiteData>::new();

    // define base image, script, js resources

    let styles = config
        .load_css()
        .entry(format!("{source_path}/styles/*.css"))
        .register()?;
    info!("Registered CSS Styles.");

    let images = config
        .load_images()
        .glob(format!("{source_path}/images/**/*.png"))
        .glob(format!("{source_path}/images/**/*.jpg"))
        .glob(format!("{source_path}/images/**/*.jpeg"))
        .glob(format!("{source_path}/images/**/*.avif"))
        .glob(format!("{source_path}/images/**/*.gif"))
        .register()?;
    info!("Registered png, jpeg, jpg, avif, gif images.");

    let scripts = config
        .load_js()
        .entry(format!("{source_path}/js/*.js"))
        .bundle(true)
        .minify(true)
        .register()?;
    info!("Registered JS Scripts. Minification and Bundling is enabled.");

    let icons = config
        .task()
        .name("Load SVG")
        .glob(format!("{source_path}/assets/**/*.svg"));
    info!("Registered SVG icons.");

    // load site posts

    let members = config
        .load_documents::<MemberMeta>()
        .source(format!("{source_path}/works/[!_]*.md"))
        .register()?;
    info!("Registered Member Pages. メンバーページを登録しました。");

    let works = config
        .load_documents::<WorkMeta>()
        .source(format!("{source_path}/works/[!_]*.md"))
        .register()?;
    info!("Registered Works Pages. 作品ページを登録しました。");

    let albums = config
        .load_documents::<AlbumMeta>()
        .source(format!("{source_path}/works/[!_]*.md"))
        .register()?;
    info!("Registered Album Pages. アルバムページを登録しました。");

    let news = config
        .load_documents::<NewsMeta>()
        .source(format!("{source_path}/works/[!_]*.md"))
        .register()?;
    info!("Registered News Pages. ニュースページを登録しました。");

    // build SiteMap

    let sitemap = config.task().using((members, works, albums, news)).merge(
        |_, (mems, works, albs, newses)| {
            let members = mems
                .values()
                .map(|m| (m.matter.ascii_name.clone(), m.matter))
                .collect();
            let works = works.values().map(|w| w.matter).collect();
            let albums = albs.values().map(|a| a.matter).collect();
            let news = newses.values().map(|n| n.matter).collect();

            let site_map = SiteMap {
                members,
                news,
                works,
                albums,
            };
            Ok(site_map)
        },
    );

    // start site dynamic page construction
}

pub fn build_site(build_id: u64, site_url: String) -> Result<(), hauchiwa::HauchiwaError> {
    let site_data = SiteData { build_id, site_url };
    info!("BUILD-{}: Configuring...", build_id);
    let mut website = Website::<SiteData>::config()
        .add_loaders([
            // load site content
            loader::glob_content(
                site_root(),
                "members/[!_]*.md",
                parse_front_matter_and_fetch_contents::<MemberMeta>,
            ),
            loader::glob_content(
                site_root(),
                "posts/[!_]*.md",
                parse_post_meta,
            ),
            loader::glob_content(site_root(), "works/[!_]*.md", parse_work_meta),
            loader::glob_content(
                site_root(),
                "albums/[!_]*.md",
                parse_front_matter_and_fetch_contents::<AlbumMeta>,
            ),
            // load CSS
            loader::glob_styles(site_root(), "styles/*.css"),
            // load JS
            loader::glob_scripts(site_root(), "js/*.js"),
            // load images
            loader::glob_images(site_root(), "images/**/*.jpg"),
            loader::glob_images(site_root(), "images/**/*.png"),
            loader::glob_images(site_root(), "images/**/*.gif"),
            loader::glob_images(site_root(), "images/**/*.avif"),
            // SVG assets require special treatment, we dont want processing
            loader::glob_assets(site_root(), "assets/**/*.svg", |rt, data| {
                let path = rt.store(&data, "svg")?;
                Ok(SvgData { path, data })
            }),
            loader::glob_assets(site_root(), "audio/**/*.ogg", |rt, data| {
                rt.store(&data, "ogg")?;
                Ok(AudioFile {})
            })
        ])
        .add_task("STATIC: build robots", |ctx| {
            info!(
                "BUILD-{}: Starting static build",
                ctx.get_globals().data.build_id
            );
            let start_time = Instant::now();

            let robots = robots_txt()?;
            let index = markup_to_page(&ctx, "index.html", index(&ctx)?)?;
            let notfound = markup_to_page(&ctx, "404.html", notfound(&ctx)?)?;
            let join_vocadou = markup_to_page(&ctx, "join.html", join_vocadou(&ctx)?)?;

            let time_taken = start_time.elapsed();
            info!(
                "BUILD-{}: Finished static build in {}s",
                ctx.get_globals().data.build_id, time_taken.as_secs_f32()
            );

            Ok(vec![robots, index, notfound, join_vocadou])
        })
        .add_task("DYNAMIC: build all dynamic content", |ctx| {
            info!(
                "BUILD-{}: Starting dynamic content build",
                ctx.get_globals().data.build_id
            );
            let start_time = Instant::now();

            let members = ctx.glob_with_file::<Content<MemberMeta>>("members/[!_]*.md")?;
            // construct name map
            let member_ascii_to_name = members
                .iter()
                .map(|member_with_file| member_with_file.data)
                .map(|member_meta| (member_meta.meta.ascii_name.clone(), member_meta.meta.name.clone()))
                .collect::<HashMap<String, String>>();

            let works = ctx.glob_with_file::<Content<WorkMeta>>("works/[!_]*.md")?;
            info!(
                "BUILD-{}: Ensuring all names exist in works.",
                ctx.get_globals().data.build_id
            );
            for work in works.iter() {
                let file_path = &work.file.file;
                let work_meta = work.data;
                if !member_ascii_to_name.contains_key(&work_meta.meta.author) {
                    let error_str = format!("BUILD-{}: ファイル {}の内, メタデータフィルド`author`でエーラ発生: {} はメンバー中見つかりませんでした。 英語ネーム使うかどうか確認してください。", ctx.get_globals().data.build_id, file_path, &work_meta.meta.author);
                    error!("{}", &error_str);
                    return Err(RuntimeError::msg(error_str))
                }
                for collaborator in &work_meta.meta.collaborators {
                    if !member_ascii_to_name.contains_key(collaborator) {
                        let error_str = format!("BUILD-{}: ファイル {}の内, メタデータフィルド`collaborators`でエーラ発生: {} はメンバー中見つかりませんでした。 英語ネーム使うかどうか確認してください。投稿者が東大ボカロP同好会のメンバーじゃないければ、`extra_collaborators`で入れてください。", ctx.get_globals().data.build_id, file_path, &collaborator);
                    error!("{}", &error_str);
                        return Err(RuntimeError::msg(error_str))
                    }
                }
            }

            info!(
                "BUILD-{}: Ensuring all names exist in albums.",
                ctx.get_globals().data.build_id
            );

            let albums = ctx.glob_with_file::<Content<AlbumMeta>>("albums/[!_]*.md")?;
            for album in &albums {
                let file_path = &album.file.file;
                let album_meta = album.data;
                for contributor in &album_meta.meta.contributors {
                    if !member_ascii_to_name.contains_key(contributor) {
                        let error_str = format!("BUILD-{}: ファイル {}の内, メタデータフィルド`contributors`でエーラ発生: {} はメンバー中見つかりませんでした。 英語ネーム使うかどうか確認してください。投稿者が東大ボカロP同好会のメンバーじゃないければ、`extra_contributors`で入れてください。", ctx.get_globals().data.build_id, file_path, &contributor);
                    error!("{}", &error_str);
                        return Err(RuntimeError::msg(error_str))
                    }
                }
            }

            info!(
                "BUILD-{}: Ensuring all names exist in posts.",
                ctx.get_globals().data.build_id
            );

            let news = ctx.glob_with_file::<Content<NewsMeta>>("posts/[!_]*.md")?;
            for post in &news {
                let file_path = &post.file.file;
                let post_meta = post.data;

                if let Some(author) = &post_meta.meta.author
                    && !member_ascii_to_name.contains_key(author) {
                        let error_str = format!("BUILD-{}: ファイル {}の内, メタデータフィルド`author`でエーラ発生: {} はメンバー中見つかりませんでした。 英語ネーム使うかどうか確認してください。", ctx.get_globals().data.build_id, file_path, author);
                    error!("{}", &error_str);
                        return Err(RuntimeError::msg(error_str))
                    }
            }

            info!(
                "BUILD-{}: Finished all pre-build checks.",
                ctx.get_globals().data.build_id
            );


            info!(
                "BUILD-{}: Starting rendering pages.",
                ctx.get_globals().data.build_id
            );

            info!(
                "BUILD-{}: Construct: SiteMap.",
                ctx.get_globals().data.build_id
            );

            let mut sitemap = SiteMap {
                members: members.iter().map(|member| { &member.data.meta }).cloned().collect(),
                news: news.iter().map(|posts| &posts.data.meta).cloned().collect(),
                works: works.iter().map(|works| &works.data.meta).cloned().collect(),
                albums: albums.iter().map(|album| &album.data.meta).cloned().collect(),
            };
            sitemap.sort_self();
            // TODO: add "worked on albums" and "posts". 

            info!(
                "BUILD-{}: Construct: minijinja Environment.",
                ctx.get_globals().data.build_id
            );

            let mut environment = Environment::new();

            // environment.add_function("sns_link", jinja_sns_icon);
            environment.add_function("sns_embed", jinja_embed);
            environment.add_function("member", jinja_member);
            // environment.add_global("SITE", );
            add_to_environment(&mut environment);
            environment.set_unknown_method_callback(unknown_method_callback);

            info!(
                "BUILD-{}: Building member pages.",
                ctx.get_globals().data.build_id
            );
            let mut member_overview = vec![Page::html("members.html", member_overview(&ctx, &sitemap).map_err(|why| why.context("Build Member Overview /members.html"))?.into_string())];
            let mut member_detail = members.iter().map(|member_page| {
                render_metadata_and_final_page(&ctx, &environment, &sitemap, &member_ascii_to_name, member_page.data, Sections::MemberProfile, &member_page.data.meta.ascii_name, format!("members/{}.html", &member_page.data.meta.ascii_name), |ctx, meta, sitemap, namemap, content| {
                    member_detail(ctx, meta, sitemap, namemap, content)
                })
            }).collect::<Result<Vec<Page>, RuntimeError>>()?;

            info!(
                "BUILD-{}: Finished building member pages.",
                ctx.get_globals().data.build_id
            );

            info!(
                "BUILD-{}: Building work & album pages.",
                ctx.get_globals().data.build_id
            );

            let mut works_overview = vec![Page::html("works.html", works_overview(&ctx, &sitemap, &member_ascii_to_name).map_err(|why| why.context("Build Works Overview works.html"))?.into_string())];

            let mut works_detail = works.iter().map(|work_page| {
                render_metadata_and_final_page(&ctx, &environment, &sitemap, &member_ascii_to_name, work_page.data, Sections::WorksPost, &work_page.data.meta.title, format!("works/releases/{}.html", work_reference(&work_page.data.meta.title, &work_page.data.meta.author)), |ctx, meta, _, namemap, content| {
                    work_detail(ctx, meta, namemap, content)
                })
            }).collect::<Result<Vec<Page>, RuntimeError>>()?;

            info!(
                "BUILD-{}: Finished building work pages.",
                ctx.get_globals().data.build_id
            );

            let mut albums_detail = albums.iter().map(|album_page| {
                render_metadata_and_final_page(&ctx, &environment, &sitemap, &member_ascii_to_name, album_page.data, Sections::AlbumPost, &album_page.data.meta.title, format!("works/albums/{}.html", album_reference(&album_page.data.meta.title, &album_page.data.meta.front_cover)), |ctx, meta, _, namemap, content| {
                    album_detail(ctx, meta, namemap, content)
                })
            }).collect::<Result<Vec<Page>, RuntimeError>>()?;

            info!(
                "BUILD-{}: Finished building album pages.",
                ctx.get_globals().data.build_id
            );

            info!(
                "BUILD-{}: Building post pages.",
                ctx.get_globals().data.build_id
            );

            let mut post_overview = vec![Page::html("news.html", news_posts(&ctx, &sitemap, &member_ascii_to_name)?.into_string())];

            let mut posts_detail = news.iter().map(|post_page| {
                render_metadata_and_final_page(&ctx, &environment, &sitemap, &member_ascii_to_name, post_page.data, Sections::NewsPost, &post_page.data.meta.title, format!("news/{}.html", post_reference(&post_page.data.meta)), |ctx, meta, _, namemap, content| {
                    post_detail(ctx, meta, content, namemap)
                })
            }).collect::<Result<Vec<Page>, RuntimeError>>()?;

            info!(
                "BUILD-{}: Finished building post pages.",
                ctx.get_globals().data.build_id
            );

            info!(
                "BUILD-{}: Building works_list.json",
                ctx.get_globals().data.build_id
            );
            // TODO: search?
            let works_list = works.iter().enumerate().map(|(id, work)| {
                let work_meta = &work.data.meta;
                let display_name = member_ascii_to_name.get(&work_meta.author).ok_or(anyhow::Error::msg("wtf???? coudlnt find member???".to_string()))?;
                let alt_desc = work_meta.short.as_ref().unwrap_or(&work_meta.title);
                let embedded_html = match &work_meta.display {
                    work::CoverOrImage::Cover(cover) => {
                        (html! {
                            img href=(cover) alt=(alt_desc) {}
                        }).into_string()
                    },
                    work::CoverOrImage::Link(url) => embed(url.as_str())?.render().into_string(),
                    work::CoverOrImage::AudioFile(lnk) => embed(lnk)?.render().into_string(),
                };
                let fixed_html = rewrite_html(&embedded_html, rewrite_settings(&ctx.get_globals().data.site_url)).map_err(|why| anyhow::Error::msg(why.to_string()))?;
                let site_url = &ctx.get_globals().data.site_url;
                Ok(DisplayWorkMeta {
                    id: id as i32,
                    title: work_meta.title.clone(),
                    description: work_meta.short.clone(),
                    on_site_link: rewrite_link(site_url, format!("/works/releases/{}.html", work_reference(&work_meta.title, &work_meta.author)))?,
                    author_displayname: display_name.clone(),
                    author_link: rewrite_link(site_url, format!("/members/{}.html", work_meta.author))?,
                    embed_html: fixed_html,
                })
            }).collect::<Result<Vec<DisplayWorkMeta>, anyhow::Error>>().map_err(|why| {
                RuntimeError::msg(why.to_string()).context("making works_list.json")
            })?;
            let works_list_serialize = serde_json::to_string(&works_list).map_err(|why| {
                RuntimeError::msg(why.to_string()).context("serializing works_list.json")
            })?;
            let mut work_list_json = vec![Page::text("works_list.json", works_list_serialize)];
            info!(
                "BUILD-{}: Finished building works_list.json",
                ctx.get_globals().data.build_id
            );

            info!(
                "BUILD-{}: Collecting pages...",
                ctx.get_globals().data.build_id
            );

            let all_lengths = member_overview.len() + member_detail.len() + works_overview.len() + works_detail.len() + albums_detail.len() + post_overview.len() + posts_detail.len() + work_list_json.len();
            let mut all_pages = Vec::with_capacity(all_lengths);
            all_pages.append(&mut member_overview);
            all_pages.append(&mut member_detail);
            all_pages.append(&mut works_overview);
            all_pages.append(&mut works_detail);
            all_pages.append(&mut albums_detail);
            all_pages.append(&mut post_overview);
            all_pages.append(&mut posts_detail);
            all_pages.append(&mut work_list_json);

            info!(
                "BUILD-{}: Running final HTML rewrite.",
                ctx.get_globals().data.build_id
            );

            let rewritten_pages = all_pages.into_iter().map(|page| {
                rewrite_page(&ctx, page)
            }).collect::<Result<Vec<Page>, RuntimeError>>()?;

            let time_taken = start_time.elapsed();
            info!(
                "BUILD-{}: Finished build phase. {} pages, took {}s.",
                ctx.get_globals().data.build_id, all_lengths, time_taken.as_secs_f32()
            );

            Ok(rewritten_pages)
        })
        .finish();
    info!("BUILD-{}: Starting build...", build_id);
    website.build(site_data)
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    set_external_bin_url(args.external_url_root.to_string());
    set_site_root(
        // args.data_root
        //     .to_str()
        //     .expect("Invalid SITE_ROOT path!")
        //     .to_string(),
        "a".to_string(),
    );
    // set_site_url(args.site_url.to_string());
    set_site_url(".".to_string());

    build_site(args.build_id, ".".to_string()).expect("Failed to build site!")
}
