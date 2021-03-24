use crate::Op;
use crate::conf::Repo;
use crate::op::autoremove::autoremovable;
use crate::unwrap_result_error;

pub fn main(
    o: &Op,
    repo: &mut Repo,
    purge: bool,
) -> bool {
    let mut modified = false;

    loop {
        let mut repeat = false;

        for id in autoremovable(&repo.addons) {
            eprintln!("Autoremove: {}{}",repo.addons.get(&id).unwrap().slug,o.suffix());
            if !o.noop {
                {
                    let addon = repo.addons.get_mut(&id).unwrap();
                    unwrap_result_error!(
                        addon.installed.as_mut().unwrap().remove(),
                        |e|"Failed to remove addon: {}",e
                    );
                    addon.installed = None;
                }
                if purge {
                    repo.addons.remove(&id);
                }
                modified = true;
                repeat = true;
            } else {
                repo.addons.remove(&id);
                repeat = true;
            }
        }

        if !repeat {break}
    }

    if o.noop {
        return false;
    }

    modified
}
