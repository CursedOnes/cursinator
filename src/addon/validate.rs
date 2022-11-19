use std::io::{Read, ErrorKind, BufRead};

use sha1::{Sha1, Digest};

use crate::addon::download::file_read;
use crate::api::API;
use crate::conf::Conf;
use crate::util::fs::Finalize;

use super::download::FilePaths;
use super::files::AddonFile;

impl AddonFile {
    pub fn validate(&self, paths: &FilePaths, cache_only: bool) -> Result<ValidateResult,anyhow::Error> {
        let file = if cache_only {&paths.cache_path} else {&paths.path};

        let mut result = ValidateResult {
            sha: String::new(),
            file_exist: file.is_file(),
            file_valid: false,
            urltxt_exist: paths.url_txt_path.is_file(),
            urltxt_valid: false,
        };

        let mut file_hash = self.sha1_hash.clone();

        if result.urltxt_exist && !cache_only {
            // read back written .url.txt file and verify url and hash
            let url_text = {
                let mut url_txt_file = file_read(&paths.url_txt_path)?;
                let mut v = Vec::with_capacity(4096);
                url_txt_file.read_to_end(&mut v)?;
                v
            };
            let mut lines = url_text.lines();
            let line_url = lines.next().transpose()?;
            let line_hash = lines.next().transpose()?;

            if let (Some(line_url),Some(line_hash)) = (line_url,line_hash) {
                if file_hash.is_none() {
                    file_hash = Some(line_hash.trim().to_owned());
                }
                result.urltxt_valid = line_url.trim() == self.download_url.0.trim() && line_hash.trim() == file_hash.as_ref().unwrap();
            }
        }

        if file_hash.is_none() && !result.urltxt_valid {
            file_hash = Some(String::new());
        }

        if result.file_exist && self.file_length == file.metadata()?.len() {
            let mut buf = vec![0u8;65536];

            let mut mod_file = file_read(&file)?;
            
            let mut hasher = Sha1::new();

            loop {
                match mod_file.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        hasher.update(&buf[..n]);
                    }
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                    Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => break,
                    Err(e) => return Err(e.into()),
                }
            }

            let sha = hasher.finalize();
            let sha_str = hex::encode(&*sha);

            result.file_valid = &sha_str == file_hash.as_ref().unwrap();
        }

        result.sha = file_hash.unwrap();

        Ok(result)
    }

    /// validate and re-download if not valid
    pub fn validate_download(&self, paths: &FilePaths, conf: &Conf, api: &mut API, fin: &mut Vec<Finalize>, cache_only: bool) -> Result<(),anyhow::Error> {
        paths.ensure_path_dir()?;
        
        let validation = self.validate(paths, cache_only)?;

        if !validation.file_valid {
            let finalization = self.download(paths, conf, api, cache_only)?;
            fin.push(finalization);
        } else if conf.url_txt && !validation.urltxt_valid && !cache_only {
            let finalizer = self.write_url_txt(paths, conf, api, &validation.sha)?;
            fin.push(finalizer);
        }

        Ok(())
    }
}

pub struct ValidateResult {
    sha: String,
    file_exist: bool,
    file_valid: bool,
    urltxt_exist: bool,
    urltxt_valid: bool,
}
