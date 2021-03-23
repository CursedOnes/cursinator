use crate::Op;
use crate::addon::release_type::ReleaseType;
use crate::addon::rtm::ReleaseTypeMode;
use crate::api::API;
use crate::conf::Repo;
use crate::addon::AddonSlug;
use crate::print::error::unwrap_addon_info;
use crate::hard_error;
use crate::api::files::FilesResult;
use crate::util::match_str::find_to_install_version_by_key;
use crate::print::error::unwrap_match;
use crate::op::update::find_version_update;
use crate::op::install::install_mod;
use crate::addon::local::UpdateOpt;

pub fn main(
    o: &Op,
    api: &API,
    repo: &mut Repo,
    mut rt: Option<ReleaseTypeMode>,
    force: bool,
    slug: String,
    version: Option<String>,
) -> bool {
    // 1. get addon id
    let slug = AddonSlug(slug);

    let addon_info = //TODO detect if slug is a addon id
        match api.search_slug(&slug) {
            Ok(r) => unwrap_addon_info(r,&repo.conf.game_version,&repo.addons),
            Err(e) => hard_error!("Failed to find addon: {}",e),
        };

    let versions = match api.files(addon_info.id) {
        FilesResult::Ok(f) => f,
        FilesResult::NotFound => hard_error!("No online information for addon"),
        FilesResult::Error(e) => hard_error!("Failed to fetch online information: {}",e),
    };

    let channel = rt.unwrap_or(ReleaseTypeMode::new(false,false,false)); //TODO use channel from previous install

    let file;
    if let Some(version) = version {
        //TODO detect if version is a file id
        file = unwrap_match(find_to_install_version_by_key( &version, &versions,&repo.conf.game_version)).z;
    } else {
        let new = find_version_update(
            &versions,
            None,
            &repo.conf.game_version,
            channel,
            true,
        );
        match new {
            Some(a) => file = a,
            None => hard_error!("No version found to install"),
        }
    }

    install_mod(
        addon_info.id,
        file.clone(),
        force,
        addon_info.slug,
        addon_info.name,
        channel,
        UpdateOpt::All, //TODO give as arg
        true,
        None, //TODO give vb as arg
        o,
        api,
        repo,
    )
}
