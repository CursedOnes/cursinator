use std::convert::TryInto;

use furse::structures::search_query::SearchQuery;

use crate::addon::AddonSlug;

use super::*;

impl API {
    pub fn search_key(&self, key: &str, page_size: u64, off: u64) -> anyhow::Result<Vec<AddonInfo>> {
        anyhow::ensure!(!key.is_empty(), "to-search key cannot be empty");

        dark_log!("API: Search key {key}");
        
        self.search_query(&SearchQuery {
            class_id: Some(6),
            search_filter: Some(key),
            page_size: Some(page_size as usize),
            index: off as usize,
            ..Default::default()
        })
    }

    pub fn search_query(&self, query: &SearchQuery) -> anyhow::Result<Vec<AddonInfo>> {
        if self.offline {hard_error!("Offline mode")};

        match self.handle_retry(|| block_on(self.furse.search_mods(&query)) ) {
            Ok(mod_files) => {Ok(
                mod_files.into_iter()
                    .filter(|addon| addon.allow_mod_distribution == Some(true) )
                    .map(|addon| {
                        AddonInfo {
                            id: AddonID(addon.id.try_into().unwrap()),
                            name: addon.name,
                            slug: AddonSlug(addon.slug),
                            summary: addon.summary,
                            latest_files_indexes: addon.latest_files_indexes,
                        }
                    })
                    .collect()
            )},
            Err(e) if e.is_response_status() == Some(reqwest::StatusCode::NOT_FOUND) => panic!("Search returns 404"),
            Err(e) => Err(e.into()),
        }
    }

    pub fn search_slug(&self, slug: &AddonSlug) -> anyhow::Result<Result<AddonInfo,Vec<AddonInfo>>> {
        anyhow::ensure!(!slug.0.is_empty(), "to-search slug cannot be empty");

        dark_log!("API: Search slug {}",slug.0);

        match self._search_slug(slug,0,50) {
            // Ok(Err(e)) => {
            //     if e.len() < 50 {return Ok(Err(e));}
            //     let mut i = 50;
            //     loop{
            //         if i >= 200 {return Ok(Err(e));}
            //         match self._search_slug(slug,i,50) {
            //             Ok(Ok(v)) => return Ok(Ok(v)),
            //             Ok(Err(v)) if v.is_empty() || v.len() < 50 => return Ok(Err(e)),
            //             Ok(Err(_)) => {},
            //             Err(e) => return Err(e),
            //         }
            //         i += 50;
            //     }
            // }
            v => v,
        }
    }
    fn _search_slug(&self, slug: &AddonSlug, page_off: u64, page_size: u64) -> anyhow::Result<Result<AddonInfo,Vec<AddonInfo>>> {
        let mut s = self.search_query(&SearchQuery {
            class_id: Some(6),
            slug: Some(slug.0.trim()),
            page_size: Some(page_size as usize),
            index: page_off as usize,
            ..Default::default()
        })?;
        let i = s.iter().enumerate()
            .find(|(_,s)| &s.slug == slug )
            .map(|(i,_)| i );
        match i {
            Some(i) => Ok(Ok(s.swap_remove(i))),
            None => Ok(Err(s)),
        }
    }
}
