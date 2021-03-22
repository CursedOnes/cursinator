use crate::Op;
use crate::conf::Repo;
use crate::op::remove::has_dependents;
use crate::print::error::unwrap_addon_match;
use crate::util::match_str::find_installed_mod_by_key;
use crate::{error,warn,unwrap_result_error};

pub fn main(
    o: &Op,
    repo: &mut Repo,
    force: bool,
    cleanup_only: bool,
    addon: String,
) -> bool {
    let addon_id = unwrap_addon_match(find_installed_mod_by_key(&addon,&repo.addons)).z;

    if cleanup_only {
        todo!() //should we error or just warn?
    } else {
        let dependents = has_dependents(addon_id, &repo.addons);

        if !dependents.is_empty() {
            if !force {
                error!("Addon has dependents:{}",o.suffix());
            } else {
                warn!("Purging Addon with dependents:{}",o.suffix());
            }

            for d in dependents {
                eprint!(" {}",d.slug);
            }
            eprintln!("{}",o.suffix());

            if !force {
                std::process::exit(1);
            }
        }

        if !o.noop {
            let addon = repo.addons.get_mut(&addon_id).unwrap();
            unwrap_result_error!(
                addon.installed.as_mut().unwrap().remove(),
                |e|"Failed to purge addon: {}",e
            );
            repo.addons.remove(&addon_id);
            return true;
        }
    }
    false
}
