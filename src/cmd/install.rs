use crate::Op;
use crate::addon::release_type::ReleaseType;
use crate::addon::rtm::ReleaseTypeMode;
use crate::api::API;
use crate::conf::Repo;
use crate::addon::AddonSlug;
use crate::print::error::unwrap_addon_info;
use crate::hard_error;

pub fn main(
    o: &Op,
    api: &API,
    repo: &mut Repo,
    rt: Option<ReleaseTypeMode>,
    force: bool,
    slug: String,
    version: Option<String>,
) -> bool {
    // 1. get addon id
    let slug = AddonSlug(slug);

    let addon_info =
    match api.search_slug(&slug) {
        Ok(r) => unwrap_addon_info(r,&repo.conf.game_version,&repo.addons),
        Err(e) => hard_error!("Failed to find addon: {}",e),
    };

    todo!()
}
