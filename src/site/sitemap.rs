use std::cmp::Ordering;
use std::collections::HashMap;

use crate::site::album::AlbumMeta;
use crate::site::member::MemberMeta;
use crate::site::news::NewsMeta;
use crate::site::work::WorkMeta;

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct MemberRef(pub String);

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct NewsRef(pub String);

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct WorkRef(pub String);

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct AlbumRef(pub String);

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct MemberDisplayName(pub String);

#[derive(Clone, Debug)]
pub struct SiteMap {
    pub members: HashMap<MemberRef, MemberMeta>,
    pub news: HashMap<NewsRef, NewsMeta>,
    pub works: HashMap<WorkRef, WorkMeta>,
    pub albums: HashMap<AlbumRef, AlbumMeta>,
    // pub member_name_to_ascii: HashMap<MemberDisplayName, MemberRef>,
    // pub member_ascii_to_name: HashMap<MemberRef, MemberDisplayName>,
}

impl SiteMap {
    // pub fn sort_self(&mut self) {
    //     self.members.sort_by(|a, b| {
    //         let a_str = a.position.clone().unwrap_or_default();
    //         let b_str = b.position.clone().unwrap_or_default();

    //         if a_str == "代表" {
    //             return Ordering::Less;
    //         } else if b_str == "代表" {
    //             return Ordering::Greater;
    //         }

    //         if a_str == "副代表" {
    //             return Ordering::Less;
    //         } else if b_str == "副代表" {
    //             return Ordering::Greater;
    //         }

    //         if a_str == "広報" {
    //             return Ordering::Less;
    //         } else if b_str == "広報" {
    //             return Ordering::Greater;
    //         }

    //         if a.position.is_some() && b.position.is_none() {
    //             return Ordering::Less;
    //         } else if a.position.is_none() && b.position.is_some() {
    //             return Ordering::Greater;
    //         } else if a.position == b.position {
    //             return a.name.cmp(&b.name);
    //         }

    //         a.name.cmp(&b.name)
    //     });

    //     self.news.sort_by(|a, b| a.date.cmp(&b.date).reverse());
    //     self.works.sort_by(|a, b| a.date.cmp(&b.date).reverse());
    //     self.albums
    //         .sort_by(|a, b| a.release_date.cmp(&b.release_date).reverse());
    // }
}
