use hauchiwa::init_logging;

use crate::site::buildsite;

pub mod commands;
pub mod site;
pub mod sql;

fn main() {
    init_logging().unwrap();
    buildsite(
        None,
        "https://toudaivocadou.org".to_string(),
        "toudaivocadou-org".to_string(),
        false,
        false,
    )
    .unwrap();
}
