use std::path::PathBuf;

use zip_extensions::*;

pub struct Extractor {
    target_path: PathBuf
}

impl Extractor {
    pub fn new(target_path: &str) -> Extractor {
        Extractor {
            target_path: PathBuf::from(target_path)
        }
    }

    pub fn target_path(&self) -> &PathBuf {
        return &self.target_path;
    }
}

pub trait Extract {
    fn extract(&self, file_path: &PathBuf, target_path: Option<&PathBuf>) -> bool;
}

impl Extract for Extractor {
    fn extract(&self, file_path: &PathBuf, target_path: Option<&PathBuf>) -> bool {
        log::info!("trying to extract '{:?}' to '{:?}'", file_path, target_path);
        if let Some(name) = file_path.file_stem() {
            if file_path.extension().is_none() {
                log::warn!(
                    "{:?}: invalid extension - ignore {:?}",
                    name,
                    file_path.extension()
                );
                return false;
            }
            if let Some(ext) = file_path.extension() {
                if ext != "zip" {
                    log::warn!("{:?}: invalid file type - ignore {:?}", name, ext);
                    return false;
                }
            }
            if let Some(file_name) = file_path.file_name() {
                log::info!("file '{:?}' written - running next steps", file_name);
                let t = target_path.unwrap_or_else(|| &self.target_path);
                let res = zip_extract(file_path, t);
                match res {
                    Ok(_) => {
                        log::info!("{:?}: archive extracted!", name);
                        return true;
                    }
                    Err(e) => {
                        log::error!("{:?}: failed to extract archive: {}", name, e);
                        return false;
                    }
                }
            } else {
                return false;
            }
        }
        return false;
    }
}