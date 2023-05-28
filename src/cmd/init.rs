use defaults::*;

use crate::{Op, hard_assert, log_error, unwrap_or_error};
use crate::conf::*;
use crate::addon::GameVersion;
use crate::addon::local::LocalAddons;

pub fn init(
    o: &Op,
    game_version: Option<String>,
    game_version_regex: Option<String>,
){
    hard_assert!(!o.conf.exists(),"repo already exists");
    let game_version = unwrap_or_error!(game_version,"no game version (-g) defined");

    let repo = Repo {
        conf: Conf {
            game_version: GameVersion::from_string(game_version),
            url_txt: default_url_txt(),
            addon_mtime: default_addon_mtime(),
            soft_retries: default_soft_retries(),
            api_headers: default_api_headers(),
            api_domain: default_api_domain(),
            override_api_key: None,
            symlink_cache_path: None, //Some("../cursinator_mod_cache".into())
            positive_loader_filter: vec![],
            negative_loader_filter: vec![],
        },
        addons: LocalAddons(Default::default()),
    };

    if !o.noop {
        log_error!(repo.save_new(&o.conf),|e|"Failed to write repo json: {}",e);
    }
}
