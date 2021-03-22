use crate::Op;
use crate::conf::Repo;
use crate::op::autoremove::autoremovable;
use crate::unwrap_result_error;

pub fn main(
    o: &Op,
    repo: &mut Repo,
    purge: bool,
) -> bool {
    for id in autoremovable(&repo.addons) {
        eprintln!("Autoremove: {}{}",repo.addons.get(&id).unwrap().slug,o.suffix());
        if !o.noop {
            {
                let addon = repo.addons.get_mut(&id).unwrap();
                unwrap_result_error!(
                    addon.installed.as_mut().unwrap().remove(),
                    |e|"Failed to remove addon: {}",e
                );
            }
            if purge {
                repo.addons.remove(&id);
            }
            return true;
        }
    }

    false
}
