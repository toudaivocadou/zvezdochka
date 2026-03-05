use maud::{Markup, html};

pub fn member(to_link: &str) -> Markup {
    html! {
        a href=(format!("/members/{to_link}"))  {
            (to_link)
        }
    }
}

pub fn jinja_member(to_link: &str) -> String {
    member(to_link).into_string()
}
