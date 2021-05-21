use crate::addon::AddonID;
use crate::addon::files::AddonFile;

use super::*;

impl API {
    pub fn files(&self, id: AddonID) -> FilesResult {
        let url = format!("{domain}/addon/{id}/files",id=id.0,domain=self.domain);
        let resp =
        match self.http_get(&url) {
            Ok(s) => s,
            Err(ureq::Error::Status(404,_)) => return FilesResult::NotFound,
            Err(e) => return FilesResult::Error(e.into()),
        };
        match resp.into_json::<Vec<AddonFile>>() {
            Ok(mut v) => {
                v.sort_unstable_by_key(|v| v.id.0 );
                FilesResult::Ok(v)
            },
            Err(e) => FilesResult::Error(e.into())
        }
    }
}

pub enum FilesResult {
    Ok(Vec<AddonFile>),
    NotFound,
    Error(anyhow::Error),
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
