use std::borrow::Borrow;
use std::convert::TryInto;

use furse::structures::file_structs::{File, HashAlgo};
use serde_derive::*;

use crate::error;

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
    pub download_url: Option<DownloadURL>,
    pub is_alternate: bool,
    pub alternate_file_id: u64,
    pub dependencies: Dependencies,
    pub is_available: bool, //TODO handle is_available
    pub package_fingerprint: u32,
    pub game_version: Vec<FileGameVersion>,
    pub has_install_script: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1_hash: Option<String>,
}

#[derive(Deserialize,Serialize,Clone)]
#[serde(transparent)]
#[repr(transparent)]
pub struct DownloadURL(pub String); //TODO TrimmedString

#[allow(dead_code)]
fn assert_memsize(a: AddonFile) -> [u8;200] {
    unsafe{
        std::mem::transmute(a)
    }
}
#[allow(dead_code)]
fn assert_omemsize(a: Option<AddonFile>) -> [u8;200] {
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

impl From<File> for AddonFile {
    fn from(file: File) -> Self {
        if file.download_url.is_none() {
            error!("{} is undistributable",file.file_name);
        }

        Self {
            id: FileID(file.id.try_into().unwrap()),
            display_name: file.display_name,
            file_name: file.file_name,
            file_date: format!("{}Z",file.file_date.to_rfc3339().split('+').next().unwrap()),
            file_length: file.file_length as u64,
            release_type: file.release_type.into(),
            is_available: file.is_available && file.download_url.is_some(),
            download_url: file.download_url.map(|d| DownloadURL(d.to_string()) ),
            is_alternate: file.expose_as_alternative == Some(true),
            alternate_file_id: file.alternate_file_id.unwrap_or(0).try_into().unwrap(),
            dependencies: file.dependencies.into(),
            package_fingerprint: file.file_fingerprint as u32,
            game_version: file.game_versions.into_iter().map(FileGameVersion).collect(),
            has_install_script: false, //TODO
            sha1_hash: file.hashes.into_iter().find(|h| h.algo == HashAlgo::Sha1 ).map(|h| h.value ),
        }
    }
}
