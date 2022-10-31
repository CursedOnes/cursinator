use std::time::{Duration, SystemTime};

use crate::addon::release_type::ReleaseType;
use crate::addon::{AddonID, AddonSlug, FileGameVersion, FileID, GameVersion};
use crate::conf::defaults::{default_api_domain, default_api_headers};
use crate::{dark_log, hard_error, warn, error};

pub mod search;
pub mod files;

use furse::Furse;
use furse::structures::file_structs::FileIndex;
use futures::executor::block_on;
use serde_derive::*;
use ureq::Agent;

pub struct API {
    // pub domain: String,
    pub agent: Agent,
    pub retry_count: u32,
    pub headers: Vec<(String,String)>,
    pub furse: Furse,
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
            retry_count: 4,
            headers: default_api_headers(),
            furse: Furse::new(include_str!("../../cf_test_key").trim()),
            offline: false,
        }
    }

    pub fn addon_info(&self, id: AddonID) -> anyhow::Result<Option<AddonInfo>> {
        if self.offline {hard_error!("Offline mode")};

        dark_log!("API: Query Addon Info for {}",id.0);

        match self.handle_retry(|| block_on(self.furse.get_mod(id.0 as i32)) ) {
            Ok(addon) => {
                assert_eq!(id.0, addon.id as u64);
                if addon.allow_mod_distribution != Some(true) {
                    error!("Mod distribution not allowed: {}",addon.slug);
                    return Ok(None); //TODO handle undistributable mod error
                }
                Ok(Some(AddonInfo {
                    id,
                    name: addon.name,
                    slug: AddonSlug(addon.slug),
                    summary: addon.summary,
                    latest_files_indexes: addon.latest_files_indexes,
                }))
            },
            Err(e) if e.is_response_status() == Some(reqwest::StatusCode::NOT_FOUND) => Ok(None),
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

    fn handle_retry<T>(&self, mut f: impl FnMut() -> Result<T,furse::Error>) -> Result<T,furse::Error> {
        let mut retry_i = 0;
        loop {
            match (f)() {
                Err(e) => {
                    if e.is_response_status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                        let wait_duration = parse_retry_duration(
                            e.is_response()
                                .and_then(|resp| resp.headers().get(reqwest::header::RETRY_AFTER) )
                                .and_then(|retry| retry.to_str().ok() ),
                            4u64.pow(retry_i.min(3)),
                        );
                        error!("Too many requests, retry in {wait_duration} seconds");
                        std::thread::sleep(Duration::from_secs(wait_duration));
                        if retry_i < self.retry_count {
                            retry_i += 1;
                            continue;
                        }
                    }
                    return Err(e);
                }
                v => return v,
            };
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct AddonInfo {
    pub id: AddonID,
    pub name: String,
    pub slug: AddonSlug,
    pub summary: String,
    pub latest_files_indexes: Vec<FileIndex>,
}

impl AddonInfo {
    pub fn release_type(&self, game_version: &GameVersion) -> Option<ReleaseType> {
        let mut max_release_type = None;
        self.latest_files_indexes.iter()
            .filter(|g| *game_version.0 == g.game_version )
            .for_each(|g| {
                let file_release_type = ReleaseType::from(g.release_type.clone());
                if max_release_type.is_none() || file_release_type >= max_release_type.unwrap() {
                    max_release_type = Some(file_release_type);
                }
            });
        max_release_type
    }
}

pub(crate) fn parse_retry_duration(retry_after: Option<&str>, fallback: u64) -> u64 {
    if let Some(retry_after) = retry_after {
        if let Ok(wait_until) = httpdate::parse_http_date(retry_after) {
            if let Ok(wait_for) = wait_until.duration_since(SystemTime::now()) {
                return wait_for.as_secs() + 1;
            }
        } else if let Ok(secs) = retry_after.parse() {
            return secs;
        }
    }
    fallback
}
