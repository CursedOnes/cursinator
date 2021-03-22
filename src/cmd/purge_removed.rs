use crate::Op;
use crate::conf::Repo;

pub fn main(
    o: &Op,
    repo: &mut Repo,
) -> bool {
    let mut modified = false;

    repo.addons.retain(|id,addon| {
        assert_eq!(id.0,addon.id.0);
        if addon.installed.is_none() {
            eprintln!("Purging: {}{}",addon.slug,o.suffix());
            if !o.noop {
                modified = true;
                false
            } else {
                true
            }
        }else{
            true
        }
    });

    modified
}
