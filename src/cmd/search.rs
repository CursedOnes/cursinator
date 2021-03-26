use crate::{Op, hard_error};
use crate::api::API;
use crate::conf::Repo;
use crate::print::addons::print_addons_search;
use crate::print::term_h;

pub fn main(
    _: &Op,
    api: &API,
    repo: &Repo,
    mut page_size: u32,
    page_n: u32,
    addon: String,
) -> bool {
    if page_size == 0 {
        page_size = term_h().saturating_sub(4).max(16) as u32;
    }
    let page_n = page_n as u64 * page_size as u64;

    match api.search_key(&addon,page_size as u64,page_n) {
        Ok(v) => print_addons_search(v.iter(),&repo.conf.game_version,&repo.addons),
        Err(e) => hard_error!("Addon Search failed: {}",e),
    }
    false
}
