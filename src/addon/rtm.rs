use std::borrow::Borrow;
use serde_derive::*;

use super::release_type::ReleaseType;

#[derive(Deserialize,Serialize)]
pub struct ReleaseTypeMode {
    pub release: bool, //TODO warn if (false,false,false)
    pub beta: bool,
    pub alpha: bool,
}

impl ReleaseTypeMode {
    pub fn new(mut release: bool, mut beta: bool, mut alpha: bool) -> Self {
        if !(release|beta|alpha) {
            release=true;beta=true;alpha=true;
        }
        Self{release,beta,alpha}
    }
    pub fn legal(&self, r: ReleaseType) -> bool {
        match r {
            ReleaseType::Release => self.alpha|self.beta|self.release,
            ReleaseType::Beta    => self.alpha|self.beta,
            ReleaseType::Alpha   => self.alpha,
        }
    }
    pub fn pick_version(&self, v: &[impl Borrow<ReleaseType>]) -> Option<usize> {
        fn find_legal(v: &[impl Borrow<ReleaseType>], g: ReleaseType) -> Option<usize> {
            v.iter().enumerate()
                .find(|(_,v)| *(*v).borrow() >= g )
                .map(|(i,_)| i )
        }

        let mut r = None;

        if r.is_none() && self.release {
            r = find_legal(v, ReleaseType::Release);
        }
        if r.is_none() && self.beta {
            r = find_legal(v, ReleaseType::Beta);
        }
        if r.is_none() && self.alpha {
            r = find_legal(v, ReleaseType::Alpha);
        }

        r
    }
}
