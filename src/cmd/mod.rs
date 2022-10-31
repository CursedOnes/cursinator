use crate::addon::release_type::ReleaseType;
use crate::addon::rtm::ReleaseTypeMode;
use crate::{Op, OpCmd, error, hard_error, log_error};
use crate::util::match_str::match_str;
use crate::conf::Repo;
use crate::api::{API, LazyFurse};
use crate::dark_log;

pub mod aset;
pub mod rset;
pub mod init;
pub mod install;
pub mod update;
pub mod channel;
pub mod list;
pub mod updates;
pub mod update_all;
pub mod remove;
pub mod auto_remove;
pub mod purge;
pub mod purge_removed;
pub mod disable;
pub mod enable;
pub mod search;

pub fn main(o: Op) {
    if let OpCmd::Init { game_version, game_version_regex } = o.cmd.clone() {
        return init::init(&o,game_version,game_version_regex);
    }
    
    let mut repo = match Repo::load(&o.conf) {
        Ok(Some(r)) => r,
        Ok(None) => hard_error!("Repo not found. set -c for repo json or initialize with init"),
        Err(e) => hard_error!("Failed to read repo json: {}",e),
    };

    let mut api = API {
        agent: ureq::Agent::new(),
        retry_count: repo.conf.soft_retries.max(1),
        //domain: repo.conf.api_domain.clone(),
        headers: repo.conf.api_headers.clone(),
        offline: o.offline,
        furse: LazyFurse::new(&repo.conf),
    };

    let modified =
    match o.cmd.clone() {
        OpCmd::Init { .. } => unreachable!(),
        OpCmd::Install { alpha, beta, release, force, addons, version_blacklist } => {
            let mut modified = false;
            for a in addons {
                match install::main(&o,&mut api,&mut repo,ReleaseTypeMode::new2(release,beta,alpha),force,a,version_blacklist.clone()) {
                    Ok(v) => modified |= v,
                    Err(e) => error!("Error installing mod: {}",e),
                }
            }
            modified
        },
        OpCmd::Search { page_size, page_n, addon } =>
            search::main(&o,&mut api,&repo,page_size,page_n,addon),
        OpCmd::Update { alpha, beta, release, allow_downgrade, force, addon, file } => 
            update::main(&o,&mut api,&mut repo,ReleaseTypeMode::new2(release,beta,alpha),allow_downgrade,force,addon,file),
        OpCmd::Channel { addon, value } => 
            channel::main(&o,&mut repo,addon,value),
        OpCmd::List {} => 
            list::main(&o,&repo),
        OpCmd::Updates { alpha, beta, release, show_all,older, addon } => 
            updates::main(&o,&mut api,&repo,ReleaseTypeMode::new2(release,beta,alpha),show_all,older,addon),
        OpCmd::UpdateAll { alpha, beta, release } => 
            update_all::main(&o,&mut api,&mut repo,ReleaseTypeMode::new2(release,beta,alpha)),
        OpCmd::Remove { force, addon } => 
            remove::main(&o,&mut repo,force,addon),
        OpCmd::AutoRemove { purge } => 
            auto_remove::main(&o,&mut repo,purge),
        OpCmd::Purge { force, cleanup_only, addon } => 
            purge::main(&o,&mut repo,force,cleanup_only,addon),
        OpCmd::PurgeRemoved {} => 
            purge_removed::main(&o,&mut repo),
        OpCmd::Disable { addon, force, disable_depending } =>
            disable::main(&o,&mut repo,addon),
        OpCmd::Enable { addon } => 
            enable::main(&o,&mut repo,addon),
        OpCmd::Aset { addon, key, value } => 
            aset::main(&o,&mut repo,addon,key,value),
        OpCmd::Rset { key, value } => 
            rset::main(&o,&mut repo,key,value),
    };

    if modified {
        dark_log!("Write repo json");
        repo.sort_deps();
        log_error!(repo.save(&o.conf),|e|"Failed to write repo json: {}",e);
    }
}

pub fn release_type_from_flags(a: bool, b: bool, r: bool) -> Option<ReleaseType> {
    if a {
        Some(ReleaseType::Alpha)
    } else if b {
        Some(ReleaseType::Beta)
    } else if r {
        Some(ReleaseType::Release)
    } else {
        None
    }
}

fn match_bool(s: &str, caption: &str) -> bool {
    let to_match = &[&[
        (false,"false"),
        (true,"true"),
        (false,"no"),
        (true,"yes"),
        (false,"0"),
        (true,"1"),
    ][..]][..];
    match match_str(s,to_match) {
        Ok(r) => r.z,
        Err(e) if e.is_empty() => hard_error!("{} must be true/false/yes/no/0/1",caption),
        Err(e) => {
            error!("Ambiguous matches for {}",caption);
            for m in e {
                m.print_error();
            }
            std::process::exit(1);
        }
    }
}
