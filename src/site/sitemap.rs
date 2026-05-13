use indexmap::IndexMap;

use crate::site::album::AlbumMeta;
use crate::site::member::MemberMeta;
use crate::site::news::NewsMeta;
use crate::site::work::WorkMeta;

pub type MemberRef = String;

#[derive(Clone, Debug)]
pub struct SiteMap {
    pub members: IndexMap<MemberRef, MemberMeta>,
    pub news: Vec<NewsMeta>,
    pub works: Vec<WorkMeta>,
    pub albums: Vec<AlbumMeta>,
    // pub member_name_to_ascii: HashMap<MemberDisplayName, MemberRef>,
    // pub member_ascii_to_name: HashMap<MemberRef, MemberDisplayName>,
}

impl SiteMap {
    pub fn find_work_by_title<S>(&self, title: &str) -> Option<&WorkMeta>
    where
        S: AsRef<str>,
    {
        let mut title_filtered_works = self
            .works
            .iter()
            .filter(|work| work.title == title)
            .collect::<Vec<&WorkMeta>>();

        return title_filtered_works.pop();
    }
}
