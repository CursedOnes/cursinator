use std::borrow::Borrow;

use serde_derive::*;

use super::{FileGameVersion, FileID};
use super::dependency::Dependencies;
use super::release_type::ReleaseType;

#[derive(Deserialize,Serialize,Clone)]
#[serde(rename_all="camelCase")]
pub struct AddonFile {
    pub id: FileID,
    pub display_name: String,
    pub file_name: String,
    pub file_date: String, //TODO serialized date
    pub file_length: u64,
    pub release_type: ReleaseType,
    pub download_url: DownloadURL,
    pub is_alternate: bool,
    pub alternate_file_id: u64,
    pub dependencies: Dependencies,
    pub is_available: bool, //TODO handle is_available
    pub package_fingerprint: u32,
    pub game_version: Vec<FileGameVersion>,
    pub has_install_script: bool,
}

#[derive(Deserialize,Serialize,Clone)]
#[serde(transparent)]
#[repr(transparent)]
pub struct DownloadURL(pub String); //TODO TrimmedString

#[allow(dead_code)]
fn assert_memsize(a: AddonFile) -> [u8;176] {
    unsafe{
        std::mem::transmute(a)
    }
}
#[allow(dead_code)]
fn assert_omemsize(a: Option<AddonFile>) -> [u8;176] {
    unsafe{
        std::mem::transmute(a)
    }
}

impl AddonFile {
    pub fn sort_deps(&mut self) {
        self.dependencies.sort_unstable_by_key(|v| (v.idx(),v.id().0) )
    }
}

impl Borrow<ReleaseType> for AddonFile {
    fn borrow(&self) -> &ReleaseType {
        &self.release_type
    }
}
