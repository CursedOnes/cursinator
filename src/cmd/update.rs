use crate::Op;
use crate::addon::release_type::ReleaseType;
use crate::api::API;
use crate::conf::Repo;

pub fn main(
    o: &Op,
    api: &API,
    repo: &mut Repo,
    rt: Option<ReleaseType>,
    force: bool,
    addon: String,
    version: Option<String>,
) -> bool {
    todo!()
}
