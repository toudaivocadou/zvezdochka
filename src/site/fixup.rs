use std::borrow::Cow;

use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};

pub fn fixup_html(build_id: Option<u64>, html: String) -> String {
    lol_html::rewrite_str(&html, settings)
}

fn build_id_to_str(build_id: u64) -> String {
    BASE64_URL_SAFE_NO_PAD.encode(format!("{build_id}"))
}

fn fixup_abs_link<'a>(build_id: Option<u64>, destination: Cow<'a, str>) -> Cow<'a, str> {
    match build_id {
        Some(build) => Cow::Owned(format!("/{}{}", build_id_to_str(build), destination)),
        None => destination,
    }
}
