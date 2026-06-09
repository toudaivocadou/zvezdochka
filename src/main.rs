use std::error::Error;

use hauchiwa::init_logging;

use crate::site::buildsite;

pub mod commands;
pub mod site;
pub mod sql;

fn main() {
    init_logging().unwrap();
    let a = buildsite(
        None,
        "https://toudaivocadou.org".to_string(),
        "toudaivocadou-org".to_string(),
        false,
        false,
    );
    if let Err(why) = a {
        println!("{:?}", why);
        println!("{:?}", why.source().unwrap());
    }
}
