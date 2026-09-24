#![deny(clippy::pedantic)]
#![warn(clippy::all)]
use crate::site::buildsite;
use hauchiwa::init_logging;
use std::error::Error;

pub mod bot;
pub mod site;

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
