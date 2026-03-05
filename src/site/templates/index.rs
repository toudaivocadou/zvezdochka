use crate::SiteData;
use crate::metadata::Metadata;
use crate::templates::base::base;
use crate::templates::partials::navbar::Sections;
use crate::util::image;
use hauchiwa::Context;
use hauchiwa::RuntimeError;
use maud::{Markup, html};

pub fn index(context: &Context<SiteData>) -> Result<Markup, RuntimeError> {
    let meta = Metadata {
        page_title: "東京大学ボカロP同好会 - University of Tokyo Vocaloid Producer Club"
            .to_string(),
        page_image: Some("images/circle-photo.jpg".to_string()),
        canonical_link: "/index.html".to_string(),
        section: Sections::Home,
        description: Some(
            "東京大学ボカロP同好会は、ボーカロイド楽曲の制作を通じて交流するサークルです。"
                .to_string(),
        ),
        author: None,
        date: None,
    };

    let content = html! {
        section #hero {
            .container {
                h2 { "ボカロ、作ろう。" }
                p { "ボーカロイド楽曲の制作を通じて交流するサークルです。" }
                a href="/join.html" .btn { "入会案内" }
            }
        }

        section #about {
            .container {
                h2 { "サークル紹介" }
                .about-content {
                    p { "東京大学ボカロP同好会は、ボーカロイド楽曲の制作を通じて交流するサークルです。" }
                    p { "週一の活動では、作曲のアイデアや課題を共有し、フィードバックし合うことで、一人では気づけなかった新しい発見があります。" }
                    p { "さらに、サークルとして活動が広がることで、楽曲がより多くの人に届くチャンスにも繋がります。" }
                    p { "まだ設立したばかりのこのサークルで、一緒に音楽を楽しみながら成長しませんか？（サークル代表　三森）"}
                }
                .about-image {
                    img .img-placeholder src=(image(context, "images/circle-photo.jpg")?) alt="サークル活動の様子" style="height: auto";
                }
            }
        }

        section #activities {
            .container {
                h2 { "活動内容" }
                .activity-list {
                    (activity("ディスコード上でのオンライン会合", "毎週土曜日 21:00〜", "作品の進捗報告や技術共有、創作のヒントなどを話し合います。"))
                    (activity("楽曲発表会", "月1回ほど", "主にオフラインで、自分の作った楽曲を共有し、部員間で作曲のノウハウを共有したり、自分の曲にフィードバックを受け取ったりします。"))
                    (activity("各種レクリエーション", "不定期", "ピクニックやボカロに関するクイズ大会などで交流を深めます。"))
                }
            }
        }

        section #featured-work {
            .container {
                h2 { "注目作品" }
                p .section-description {
                    "メンバーの作品をランダムにピックアップしてご紹介します。リロードするたびに違う作品が表示されます。"
                }
                #featured-work-container {
                    .youtube-embed-container #embed {
                        ""
                    }

                    .featured-work-info {
                        a href="" #featured-work-link {
                            h3 #featured-work-title {
                                "曲名"
                            }
                        }
                        p {
                            "制作:"
                            a #featured-work-creator href="" {
                                "メンバー名"
                            }
                        }
                        p #featured-work-description {
                            ""
                        }
                        div style="margin-top: auto;" {
                            .click-button{
                                a href="/works.html" {
                                p {
                                    "全曲一覧になる"
                                }
                            }
                        }

                        .click-button{
                            p #reload {
                                "曲をリロード"
                            }
                        }
                        }
                    }
                }
            }
        }

        // section #news {
        //     .container {
        //         h2 { "最新ニュース" }
        //         .about-content {
        //             // TODO: 最新ニュースの筋、リンク
        //         }
        //     }
        // }

    };

    base(context, &meta, Some(&["script.js"]), content)
}

fn activity(title: &str, timeframe: &str, description: &str) -> Markup {
    html! {
        .activity-item {
            h3 { (title) }
            p { (timeframe) }
            p { (description) }
        }
    }
}
