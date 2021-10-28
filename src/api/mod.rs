use crate::addon::release_type::ReleaseType;
use crate::addon::{AddonID, AddonSlug, FileGameVersion, FileID, GameVersion};
use crate::conf::defaults::{default_api_domain, default_api_headers};
use crate::{dark_log, hard_error, warn};

pub mod search;
pub mod files;

use serde_derive::*;
use ureq::Agent;

pub struct API {
    pub domain: String,
    pub agent: Agent,
    pub headers: Vec<(String,String)>,
    pub offline: bool,
}

impl API {
    pub fn http_get(&self, url: &str) -> Result<ureq::Response,ureq::Error> {
        if self.offline {hard_error!("Offline mode")};
        dark_log!("API: {}",url);
        let mut req = self.agent.get(url);
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
            agent: Agent::new(),
            domain: default_api_domain(),
            headers: default_api_headers(),
            offline: false,
        }
    }

    pub fn addon_info(&self, id: AddonID) -> anyhow::Result<Option<AddonInfo>> {
        if self.offline {hard_error!("Offline mode")};
        let url = format!("{domain}/addon/{id}",id=id.0,domain=self.domain);
        match self.http_get(&url) {
            Ok(s) => Ok(Some( s.into_json::<AddonInfo>()? )),
            Err(ureq::Error::Status(404,_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn addon_by_id_or_slug(&self, id: &AddonSlug) ->  anyhow::Result<Result<AddonInfo,Vec<AddonInfo>>> {
        if let Ok(i) = id.0.trim().parse::<u64>() {
            match self.addon_info(AddonID(i)) {
                Ok(Some(info)) => return Ok(Ok(info)),
                Ok(None) => {},
                Err(e) => warn!("{}",e),
            }
        }
        self.search_slug(id)
    }
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct AddonInfo {
    pub id: AddonID,
    pub name: String,
    pub slug: AddonSlug,
    pub summary: String,
    pub game_version_latest_files: Vec<GameVersionLatestFiles>,
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
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
