use std::rc::Rc;

use futures::executor::block_on; //TODO use reqwest::blocking in furse or rewrite to async
use rustc_hash::FxHashMap;

use crate::addon::AddonID;
use crate::addon::files::AddonFile;

use super::*;

impl API {
    pub fn files(&mut self, id: AddonID) -> FilesResult {
        if self.offline {hard_error!("Offline mode")};

        dark_log!("API: Query Addon Files for {}",id.0);

        match handle_retry(|| self.furse.get_mut().get_mod_files(id.0 as i32), self.retry_count) {
            Ok(mod_files) => {
                let mut mod_files: Vec<AddonFile> = mod_files.into_iter().map(Into::into).collect();
                mod_files.sort_unstable_by_key(|mod_file| mod_file.id.0 );
                FilesResult::Ok(mod_files)
            },
            Err(e) if e.is_response_status() == Some(furse::reqwest::StatusCode::NOT_FOUND) => FilesResult::NotFound,
            Err(e) => FilesResult::Error(Rc::new(e.into())),
        }
    }

    pub fn files_cached(&mut self, id: AddonID, cache: &mut FxHashMap<AddonID,FilesResult>) -> FilesResult {
        cache.entry(id)
            .or_insert_with(|| self.files(id) )
            .clone()
    }
}

#[derive(Clone)]
pub enum FilesResult {
    Ok(Vec<AddonFile>),
    NotFound,
    Error(Rc<anyhow::Error>),
}

#[test]
fn test_fetch_files() {
    let files = API::test_api().files(AddonID(220311));
    match files {
        FilesResult::Ok(v) => {
            assert_eq!(v[307].id.0, 3238200);
        },
        FilesResult::NotFound => panic!("no results"),
        FilesResult::Error(e) => panic!("{}",e),
    }
}
