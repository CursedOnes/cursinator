use crate::{Op, unwrap_result_error};
use crate::conf::Repo;
use crate::op::channel::decode_channel;
use crate::print::error::unwrap_match;
use crate::util::match_str::find_installed_mod_by_key;

pub fn main(
    o: &Op,
    repo: &mut Repo,
    addon: String,
    value: Option<String>,
) -> bool {
    let addon_id = unwrap_result_error!(unwrap_match(find_installed_mod_by_key(&addon,&repo.addons,true))).z;

    if let Some(value) = value {
        let new_channel = decode_channel(&value);
        let addon = repo.addons.get_mut(&addon_id).unwrap();
        eprintln!("{}: {} -> {}{}",addon.slug,addon.channel,new_channel,o.suffix());
        if !o.noop && addon.channel != new_channel {
            addon.channel = new_channel;
            return true;
        }
    }else{
        let addon = repo.addons.get(&addon_id).unwrap();
        eprintln!("{}: {}{}",addon.slug,addon.channel,o.suffix());
    }
    
    false
}
