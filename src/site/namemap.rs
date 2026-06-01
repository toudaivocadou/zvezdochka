use indexmap::IndexMap;

pub type MemberRef = String;

#[derive(Clone, Debug)]
pub struct NameMap {
    pub members: IndexMap<MemberRef, String>,
}
