use crate::util::fs::Finalize;
use crate::Op;
use crate::api::API;
use crate::conf::Repo;
use crate::error;

pub fn main(
    _: &Op,
    api: &mut API,
    repo: &Repo,
) -> bool {
    let mut finalizers: Vec<Finalize> = vec![];

    for addon_file in repo.addons.iter().filter_map(|addon| addon.1.installed.as_ref() ) {
        match addon_file.validate_download(&repo.conf, api, &mut finalizers) {
            Ok(_) => finalizers.drain(..).for_each(Finalize::finalize),
            Err(e) => error!("Failed to download addon: {}",e),
        }
    }

    false
}
