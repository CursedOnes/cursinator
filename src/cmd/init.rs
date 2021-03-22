use std::str;
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
            game_version: GameVersion(game_version),
            url_txt: default_url_txt(),
            addon_mtime: default_addon_mtime(),
            soft_retries: default_soft_retries(),
            headers: default_headers(),
            domain: default_domain(),
        },
        addons: LocalAddons(Default::default()),
    };

    log_error!(repo.save_new(&o.conf),|e|"Failed to write repo json: {}",e);
}
