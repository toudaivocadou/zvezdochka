use std::cmp::Ordering;

use crate::news::NewsMeta;
use crate::work::WorkMeta;
use crate::{album::AlbumMeta, member::MemberMeta};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SiteMap {
    pub members: Vec<MemberMeta>,
    pub news: Vec<NewsMeta>,
    pub works: Vec<WorkMeta>,
    pub albums: Vec<AlbumMeta>,
}

impl SiteMap {
    pub fn sort_self(&mut self) {
        self.members.sort_by(|a, b| {
            let a_str = a.position.clone().unwrap_or_default();
            let b_str = b.position.clone().unwrap_or_default();

            if a_str == "代表" {
                return Ordering::Less;
            } else if b_str == "代表" {
                return Ordering::Greater;
            }

            if a_str == "副代表" {
                return Ordering::Less;
            } else if b_str == "副代表" {
                return Ordering::Greater;
            }

            if a_str == "広報" {
                return Ordering::Less;
            } else if b_str == "広報" {
                return Ordering::Greater;
            }

            if a.position.is_some() && b.position.is_none() {
                return Ordering::Less;
            } else if a.position.is_none() && b.position.is_some() {
                return Ordering::Greater;
            } else if a.position == b.position {
                return a.name.cmp(&b.name);
            }

            a.name.cmp(&b.name)
        });

        self.news.sort_by(|a, b| a.date.cmp(&b.date).reverse());
        self.works.sort_by(|a, b| a.date.cmp(&b.date).reverse());
        self.albums
            .sort_by(|a, b| a.release_date.cmp(&b.release_date).reverse());
    }
}
