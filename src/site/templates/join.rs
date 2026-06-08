use hauchiwa::{Tracker, loader::Document};
use maud::{Markup, PreEscaped, html};

use crate::site::{
    namemap::NameMap,
    util::{author_list, reference},
    work::WorkMeta,
};

pub fn join_vocadou(name_map: &NameMap, works: &Tracker<'_, Document<WorkMeta>>) -> Markup {
    // let meta = Metadata {
    //     page_title: "入会希望者へ - Joining Vocaloid Producer Club".to_string(),
    //     page_image: Some("circle-photo.jpg".to_string()),
    //     canonical_link: "/join.html".to_string(),
    //     section: Sections::Join,
    //     author: None,
    //     date: None,
    //     description: None,
    // };
    let mut works = works.iter().map(|(_, m)| m).collect::<Vec<_>>();
    works.sort_by(|a, b| {
        a.matter
            .date
            .partial_cmp(&b.matter.date)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    works.reverse();
    let works = works.into_iter().map(|document| {
        html! {
            .card {
                .youtube-embed-container {}
                a href=(format!("/works/releases/{}.html", reference(&document.matter.title, &document.matter.authors, &document.matter.additional_authors ))) {
                    h6 {
                        (document.matter.title)
                    }
                }
                p { "投稿者: " (author_list(&name_map, &document.matter.authors, &document.matter.additional_authors)) }
            }
        }
    }).collect::<Vec<PreEscaped<String>>>();
    html! {
        section #hero {
            .container {
                h2 { "ボカロP同好会、入会しよう。" }
                p { "ボーカロイド楽曲の制作を通じて交流するサークルです。" }
                a href="#join" .btn { "入会案内" }
            }
        }

            section .flex-container {
                h2 { "メンバー作品" }
                #infinite-slider .carousel {
                    #visible-slider-group .group {
                        @for work in &works {
                            (work)
                        }
                    }
                    #hidden-slider-group aria-hidden .group {
                        @for work in &works {
                            (work)
                        }
                    }
                }
            }

        section #join {
            .container {
                h2 { "入会案内" }
                .join-info {
                    p { "東京大学の学生であれば、学部・学年を問わず入会できます。音楽制作の経験がなくても大歓迎です！" }
                    p { "入会を希望される方は、下記のXアカウントまでご連絡ください。" }
                    p .contact-email {
                        a href="https://twitter.com/toudaivocadou/" {
                            "@toudaivocadou"
                        }
                    }
                    p { "または、新歓期間中の説明会にお越しください。" }
                    .join-details {
                        h3 { "説明会情報" }
                        p { "日時: 4月12日 18:00〜18:30" }
                        p { "説明会の参加方法に関しましては、公式Xアカウントで随時お知らせいたします。" }
                        p { "また、日時に関しても変更される場合がありますので、公式Xアカウントからの情報を随時ご確認ください。" }
                    }
                }
            }
        }
    }
}
