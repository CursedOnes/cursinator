use crate::Op;
use crate::conf::Repo;
use crate::print::addons::print_addons_local;

pub fn main(
    o: &Op,
    repo: &Repo,
) -> bool {
    print_addons_local(&repo.addons);
    false
}
