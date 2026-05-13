use maud::{Markup, html};

pub fn notfound() -> Markup {
    html! {
        section #hero {
            h2 { "このページは見つかりませんでした。" }
        }

        section #content {
            .container {
                a href="/index.html" .back-button {
                    "メインページに一覧に戻る"
                }
            }
        }
    }
}
