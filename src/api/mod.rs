use crate::addon::release_type::ReleaseType;
use crate::addon::{AddonID, AddonSlug, FileGameVersion, FileID, GameVersion};
use crate::conf::defaults::{default_domain, default_headers};
use crate::hard_error;

pub mod search;
pub mod files;

use serde_derive::*;

pub struct API {
    pub domain: String,
    pub headers: Vec<(String,String)>,
    pub offline: bool,
}

impl API {
    pub fn http_get(&self, url: &str) -> Result<ureq::Response,ureq::Error> {
        if self.offline {hard_error!("Offline mode")};
        let mut req = ureq::get(url);
        for (h,v) in &self.headers {
            req = req.set(h,v);
        };
        let resp = req.call()?;
        assert_eq!(resp.status(),200);
        Ok(resp)
    }

    #[allow(dead_code)]
    fn test_api() -> Self {
        Self {
            domain: default_domain(),
            headers: default_headers(),
            offline: false,
        }
    }

    pub fn addon_info(&self, id: AddonID) -> anyhow::Result<Option<AddonInfo>> {
        if self.offline {hard_error!("Offline mode")};
        let url = format!("{domain}/api/v2/addon/{id}",id=id.0,domain=self.domain);
        match self.http_get(&url) {
            Ok(s) => Ok(Some( s.into_json::<AddonInfo>()? )),
            Err(ureq::Error::Status(404,_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize="camelCase"))]
pub struct AddonInfo {
    pub id: AddonID,
    pub name: String,
    pub slug: AddonSlug,
    pub summary: String,
    pub game_version_latest_files: Vec<GameVersionLatestFiles>,
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize="camelCase"))]
pub struct GameVersionLatestFiles {
    pub game_version: FileGameVersion,
    pub project_file_id: FileID,
    pub file_type: ReleaseType,
}

impl AddonInfo {
    pub fn release_type(&self, game_version: &GameVersion) -> Option<ReleaseType> {
        let mut rt = None;
        self.game_version_latest_files.iter()
            .filter(|g| *game_version == g.game_version )
            .for_each(|g| 
                if rt.is_none() || g.file_type >= rt.unwrap() {
                    rt = Some(g.file_type);
                }
            );
        rt
    }
}
