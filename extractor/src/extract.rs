use std::path::PathBuf;

use zip_extensions::*;

pub struct Extractor {
    pub target_path: PathBuf,
    pub archive_dir: PathBuf
}

impl Extractor {
    pub fn new(target_path: &str, archive_dir: &str) -> Self {
        Self {
            target_path: PathBuf::from(target_path),
            archive_dir: PathBuf::from(archive_dir)
        }
    }
}

pub trait Extract {
    fn extract(&self, file_path: &PathBuf, target_path: Option<&PathBuf>) -> bool;
}

impl Extract for Extractor {
    fn extract(&self, file_path: &PathBuf, target_path: Option<&PathBuf>) -> bool {
        let default_target = self.target_path.join(file_path.file_stem().unwrap());
        let t = target_path.unwrap_or_else(|| &default_target);
        log::info!("trying to extract '{:?}' to '{:?}'", file_path, t);
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