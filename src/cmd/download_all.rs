use crate::util::fs::Finalize;
use crate::Op;
use crate::api::API;
use crate::conf::Repo;
use crate::error;

pub fn main(
    o: &Op,
    api: &mut API,
    repo: &Repo,
    cache_only: bool,
) -> bool {
    let mut finalizers: Vec<Finalize> = vec![];

    for (&addon_id,addon) in repo.addons.iter() {
        if let Some(addon_file) = addon.installed.as_ref() {
            let paths = addon_file.file_paths_current(addon_id, !o.noop);
            match addon_file.validate_download(&paths, &repo.conf, api, &mut finalizers, cache_only) {
                Ok(_) => finalizers.drain(..).for_each(Finalize::finalize),
                Err(e) => error!("Failed to download addon: {}",e),
            }
        }
    }

    false
}
