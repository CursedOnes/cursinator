use crate::Op;
use crate::addon::release_type::ReleaseType;
use crate::addon::rtm::ReleaseTypeMode;
use crate::api::API;
use crate::conf::Repo;

pub fn main(
    o: &Op,
    api: &API,
    repo: &mut Repo,
    rt: Option<ReleaseTypeMode>,
    force: bool,
    addon: String,
    version: Option<String>,
) -> bool {
    todo!()
}
