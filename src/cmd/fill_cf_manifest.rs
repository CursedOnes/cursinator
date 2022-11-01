use serde::{Deserialize, Serialize};

use crate::addon::local::LocalAddon;
use crate::conf::Repo;
use crate::{error, warn};

fn process(manifest: &mut CfManifest, repo: &Repo) {
    if manifest.manifest_version.as_i64() != Some(1) {
        warn!("Unknown CfManifest template version ({})",manifest.manifest_version);
    }

    let mut remaining_addons: Vec<&LocalAddon> = repo.addons.values().collect();

    for entry in &mut manifest.files {
        entry.handle_entry(&mut remaining_addons);
    }

    handle_entries_post(&mut manifest.files);

    for addon in remaining_addons {
        manifest.files.push(CfMFile::auto_create(addon));
    }
}

#[derive(Deserialize,Serialize)]
pub struct CfManifest {
    #[serde(rename = "manifestVersion")]
    manifest_version: serde_json::Value,

    files: Vec<CfMFile>,

    #[serde(flatten)]
    other: serde_json::Value,
}

#[derive(Deserialize,Serialize)]
pub struct CfMFile {
    /// If set, cursinator will ignore this entry with manually filled in data.
    #[serde(skip_serializing)]
    #[serde(default)]
    cursinator_ignore: bool,
    /// Entries with cursinator_exclude will be removed, unless cursinator_ignore is set.
    /// 
    /// Combine with cursinator_slug or project_id to exclude an addon
    #[serde(skip_serializing)]
    #[serde(default)]
    cursinator_exclude: bool,
    /// If set, try to resolve slug and fill with it instead of projectID
    #[serde(skip_serializing)]
    cursinator_slug: Option<String>,

    #[serde(rename = "fileID")]
    file_id: Option<u64>,

    required: Option<bool>,

    #[serde(rename = "projectID")]
    project_id: Option<u64>,

    #[serde(flatten)]
    other: serde_json::Value,
}

impl CfMFile {
    fn handle_entry(&mut self, remaining: &mut Vec<&LocalAddon>) {
        if self.cursinator_ignore {
            // cursinatore_exclude should also be ignored if cursinator_ignore is set
            self.cursinator_exclude = false;
        } else if let Some(slug) = &self.cursinator_slug {
            // try resolve cursinator_slug and fill project_id and file
            if let Some(id) = self.project_id {
                warn!("Both cursinator_slug ({slug}) and projectID ({id}) set. cursinator_slug is prioritized");
            }
            if let Some(idx) = find_entry_by_id_or_slug(slug, remaining) {
                let addon = remaining[idx];
                remaining.swap_remove(idx);

                self.project_id = Some(addon.id.0);

                if self.required.is_none() {
                    self.required = Some(true);
                }
                if self.file_id.is_none() {
                    self.file_id = Some(addon.installed.as_ref().unwrap().id.0);
                }
            } else {
                error!("cursinator_slug not found: {slug}");
            }
        } else if let Some(id) = self.project_id {
            // fill in file if only projectID is given
            if let Some(idx) = find_entry_by_id(id, remaining) {
                let addon = remaining[idx];
                remaining.swap_remove(idx);

                if self.required.is_none() {
                    self.required = Some(true);
                }
                if self.file_id.is_none() {
                    self.file_id = Some(addon.installed.as_ref().unwrap().id.0);
                }
            } else {
                error!("projectID not bound: {id}");
            }
        }
        // whatever happens, we will remove mentioned projectID from remaining, so they won't be autofilled later
        if let Some(id) = self.project_id {
            if let Some(idx) = find_entry_by_id(id, remaining) {
                remaining.swap_remove(idx);
            }
        }
    }

    fn auto_create(addon: &LocalAddon) -> Self {
        Self {
            cursinator_ignore: true,
            cursinator_exclude: false,
            cursinator_slug: None,
            file_id: Some(addon.installed.as_ref().unwrap().id.0),
            required: Some(true),
            project_id: Some(addon.id.0),
            other: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

fn handle_entries_post(v: &mut Vec<CfMFile>) {
    v.retain(|v| !v.cursinator_exclude );
}

fn find_entry_by_id_or_slug(v: &str, list: &[&LocalAddon]) -> Option<usize> {
    if let Ok(i) = v.parse::<u64>() {
        if let Some(i) = find_entry_by_id(i, list) {
            return Some(i);
        }
    }
    list.iter().enumerate().find(|(_,addon)| addon.slug.0.trim() == v.trim() ).map(|(i,_)| i )
}

fn find_entry_by_id(v: u64, list: &[&LocalAddon]) -> Option<usize> {
    list.iter().enumerate().find(|(_,addon)| addon.id.0 == v ).map(|(i,_)| i )
}
