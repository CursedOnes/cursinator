use crate::addon::AddonSlug;

use super::*;

impl API {
    pub fn search_key(&self, key: &str, page_size: u64, off: u64) -> anyhow::Result<Vec<AddonInfo>> {
        anyhow::ensure!(key.len() != 0, "to-search key cannot be empty");
        if self.offline {hard_error!("Offline mode")};
        let url = format!(
            "{domain}/api/v2/addon/search?gameId=432&index={off}&pageSize={page_size}&sectionId=6&searchFilter={key}",
            off=off,
            page_size=page_size,
            key=key,
            domain=self.domain,
        );
        let resp = self.http_get(&url)?;
        let res = resp.into_json::<Vec<AddonInfo>>()?;
        Ok(res)
    }
    pub fn search_slug(&self, slug: &AddonSlug) -> anyhow::Result<Result<AddonInfo,Vec<AddonInfo>>> {
        anyhow::ensure!(slug.0.len() != 0, "to-search slug cannot be empty");
        match self._search_slug(slug,0,64) {
            Ok(Err(e)) => {
                if e.len() < 64 {return Ok(Err(e));}
                let mut i = 0;
                loop{
                    match self._search_slug(slug,i,10000) {
                        Ok(Ok(v)) => return Ok(Ok(v)),
                        Ok(Err(v)) if v.is_empty() || v.len() < 5000 => return Ok(Err(e)),
                        Ok(Err(_)) => {},
                        Err(e) => return Err(e),
                    }
                    i += 5000;
                }
            }
            v => v,
        }
    }
    fn _search_slug(&self, slug: &AddonSlug, page_off: u64, page_size: u64) -> anyhow::Result<Result<AddonInfo,Vec<AddonInfo>>> {
        let mut s = self.search_key(slug.0.trim(),page_size,page_off)?;
        let i = s.iter().enumerate()
            .find(|(_,s)| &s.slug == slug )
            .map(|(i,_)| i );
        match i {
            Some(i) => Ok(Ok(s.swap_remove(i))),
            None => Ok(Err(s)),
        }
    }
}
