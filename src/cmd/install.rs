use anyhow::bail;

use crate::{Op, unwrap_result_error};
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
    api: &mut API,
    repo: &mut Repo,
    rt: Option<ReleaseTypeMode>,
    force: bool,
    addon_query: String,
    version_blacklist: Option<String>,
) -> Result<bool,anyhow::Error> {
    let (slug,version) = unwrap_result_error!(
        decode_name_version(&addon_query),
        |e| "Failed to decode addon query: {}",e
    );

    // 1. get addon id
    let slug = AddonSlug(slug);

    let addon_info = //TODO detect if slug is a addon id
        match api.addon_by_id_or_slug(&slug) {
            Ok(r) => unwrap_addon_info(r,&repo.conf.game_version,&repo.addons),
            Err(e) => bail!("Failed to find addon: {}",e),
        };

    let versions = match api.files(addon_info.id) {
        FilesResult::Ok(f) => f,
        FilesResult::NotFound => bail!("No online information for addon"),
        FilesResult::Error(e) => bail!("Failed to fetch online information: {}",e),
    };

    if !versions.iter().any(|v| repo.conf.game_version.matches(v.game_version.iter()) ) {
        bail!("No version for current game version");
    }

    let channel = rt.unwrap_or_else(|| ReleaseTypeMode::new(false,false,false) ); //TODO use channel from previous install

    let file;
    if let Some(version) = version {
        //TODO detect if version is a file id
        file = unwrap_match(find_to_install_version_by_key( &version, &versions,&repo.conf.game_version))?.z;
    } else {
        let new = find_version_update(
            &versions,
            None,
            &repo.conf.game_version,
            version_blacklist.as_deref(),
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

fn decode_name_version(mut mod_req: &str) -> Result<(String,Option<String>),anyhow::Error> {
    if let Some((v,_)) = mod_req.split_once('?') {
        mod_req = v;
    }

    if let Some((_,v)) = mod_req.split_once("curseforge.com/minecraft/mc-mods/") {
        mod_req = v;
    }
    if let Some((_,v)) = mod_req.split_once("curseforge.com/projects/") {
        mod_req = v;
    }

    if mod_req.contains("://") {
        bail!("Invalid pattern {mod_req}");
    }
    
    let mut slug_result = mod_req;
    let mut version_result = None;

    if let Some((slug_part,mut version_part)) = mod_req.split_once('/') {
        slug_result = slug_part;
        if let Some(v) = version_part.strip_prefix("files/all") {
            version_part = v;
        }
        if let Some(v) = version_part.strip_prefix("files") {
            version_part = v;
        }
        if let Some(v) = version_part.strip_prefix("download") {
            version_part = v;
        }
        while let Some(v) = version_part.strip_prefix('/') {
            version_part = v;
        }
        if let Some((v,_)) = version_part.split_once('/') {
            version_part = v;
        }
        if !version_part.is_empty() {
            version_result = Some(version_part);
        }
    } else if let Some((slug_part,version_part)) = mod_req.split_once('=') {
        slug_result = slug_part;
        version_result = Some(version_part);
    } else if let Some((slug_part,version_part)) = mod_req.split_once("@") {
        slug_result = slug_part;
        version_result = Some(version_part);
    }

    while let Some(v) = slug_result.strip_suffix('/') {
        slug_result = v;
    }
    if let Some(v) = slug_result.strip_suffix("/files/all") {
        slug_result = v;
    }
    if let Some(v) = slug_result.strip_suffix("/files") {
        slug_result = v;
    }
    if let Some(v) = slug_result.strip_suffix("/download") {
        slug_result = v;
    }

    if let Some(version_result) = &mut version_result {
        while let Some(v) = version_result.strip_suffix('/') {
            *version_result = v;
        }
    }

    Ok((slug_result.to_owned(),version_result.map(ToOwned::to_owned)))
}

#[test]
fn test_pation() {
    assert_eq!(
        decode_name_version("foo").unwrap(),
        ("foo".to_owned(),None)
    );
    assert_eq!(
        decode_name_version("https://www.curseforge.com/minecraft/mc-mods/tinkers-construct/files").unwrap(),
        ("tinkers-construct".to_owned(),None)
    );
    assert_eq!(
        decode_name_version("tinker=5.0.0").unwrap(),
        ("tinker".to_owned(),Some("5.0.0".to_owned()))
    );
    assert_eq!(
        decode_name_version("tinker@5.0.0").unwrap(),
        ("tinker".to_owned(),Some("5.0.0".to_owned()))
    );
    assert_eq!(
        decode_name_version("tinker/5.0.0").unwrap(),
        ("tinker".to_owned(),Some("5.0.0".to_owned()))
    );
    assert_eq!(
        decode_name_version("https://www.curseforge.com/minecraft/mc-mods/tinkers-construct/files/all").unwrap(),
        ("tinkers-construct".to_owned(),None)
    );
    assert_eq!(
        decode_name_version("https://www.curseforge.com/minecraft/mc-mods/tinkers-construct/files/all?filter-game-version=2020709689%3A7498").unwrap(),
        ("tinkers-construct".to_owned(),None)
    );
    assert_eq!(
        decode_name_version("https://www.curseforge.com/minecraft/mc-mods/tinkers-construct/files/3998764").unwrap(),
        ("tinkers-construct".to_owned(),Some("3998764".to_owned()))
    );
    assert_eq!(
        decode_name_version("https://www.curseforge.com/minecraft/mc-mods/tinkers-construct/files/3998764/").unwrap(),
        ("tinkers-construct".to_owned(),Some("3998764".to_owned()))
    );
    assert_eq!(
        decode_name_version("https://www.curseforge.com/minecraft/mc-mods/tinkers-construct/download/3998764").unwrap(),
        ("tinkers-construct".to_owned(),Some("3998764".to_owned()))
    );
}
