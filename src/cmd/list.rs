use crate::Op;
use crate::addon::local::LocalAddon;
use crate::conf::Repo;
use crate::print::addons::print_addons_local;

pub fn main(
    _: &Op,
    repo: &Repo,
) -> bool {
    let mut addons: Vec<&LocalAddon> = repo.addons.values().collect();
    addons.sort_unstable_by_key(|a| &a.slug.0 );
    print_addons_local(addons.into_iter());
    false
}
