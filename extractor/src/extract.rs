use sevenz_rust::Error;
use std::{
    fs::File,
    path::{Path, PathBuf},
};
use zip::result::ZipError;

pub struct Extractor {
    pub target_path: PathBuf,
    pub archive_dir: PathBuf,
}

impl Extractor {
    pub fn new(target_path: &str, archive_dir: &str) -> Self {
        Self {
            target_path: PathBuf::from(target_path),
            archive_dir: PathBuf::from(archive_dir),
        }
    }
}

pub trait Extract {
    fn extract(&self, file_path: &PathBuf, target_path: Option<&Path>) -> Result<bool, String>;
}

impl Extract for Extractor {
    fn extract(&self, file_path: &PathBuf, target_path: Option<&Path>) -> Result<bool, String> {
        let mut ext = "";
        let t = target_path
            .unwrap_or_else(|| &self.target_path)
            .canonicalize()
            .expect("failed to canonicalize target path")
            .join(file_path.file_stem().unwrap());
        log::info!("trying to extract '{:?}' to '{:?}'", file_path, t);
        if let Some(name) = file_path.file_stem() {
            if file_path.extension().is_none() {
                log::warn!(
                    "{:?}: invalid extension - ignore {:?}",
                    name,
                    file_path.extension()
                );
                return Ok(false);
            }
            if let Some(e) = file_path.extension() {
                ext = e.to_str().unwrap();
                if ext != "zip" && ext != "7z" {
                    log::warn!("{:?}: invalid file type - ignore {:?}", name, ext);
                    return Ok(false);
                }
            }
            return if let Some(file_name) = file_path.file_name() {
                log::info!("file '{:?}' written - running next steps", file_name);
                let file = File::open(file_path).expect("failed to open file");

                let mut res: Result<(), String> = Err("invalid".to_string());
                if ext == "zip" {
                    let archive = zip::ZipArchive::new(file);
                    if archive.is_err() {
                        return Err(archive.err().unwrap().to_string());
                    }
                    res = archive.unwrap().extract(t).or_else(|e| match e {
                        ZipError::Io(_) => Err("Io".to_string()),
                        ZipError::InvalidArchive(_) => Err("InvalidArchive".to_string()),
                        ZipError::UnsupportedArchive(_) => Err("UnsupportedArchive".to_string()),
                        ZipError::FileNotFound => Err("FileNotFound".to_string()),
                        ZipError::InvalidPassword => Err("InvalidPassword".to_string()),
                        _ => Err("Unknown".to_string()),
                    })
                } else if ext == "7z" {
                    res = sevenz_rust::decompress(file, t).or_else(|e| match e {
                        Error::BadSignature(_) => Err("BadSignature".to_string()),
                        Error::UnsupportedVersion { .. } => Err("UnsupportedVersion".to_string()),
                        Error::ChecksumVerificationFailed => {
                            Err("ChecksumVerificationFailed".to_string())
                        }
                        Error::NextHeaderCrcMismatch => Err("NextHeaderCrcMismatch".to_string()),
                        Error::Io(_, _) => Err("Io".to_string()),
                        Error::FileOpen(_, _) => Err("FileOpen".to_string()),
                        Error::Other(_) => Err("Other".to_string()),
                        Error::BadTerminatedStreamsInfo(_) => {
                            Err("BadTerminatedStreamsInfo".to_string())
                        }
                        Error::BadTerminatedUnpackInfo => {
                            Err("BadTerminatedUnpackInfo".to_string())
                        }
                        Error::BadTerminatedPackInfo(_) => Err("BadTerminatedPackInfo".to_string()),
                        Error::BadTerminatedSubStreamsInfo => {
                            Err("BadTerminatedSubStreamsInfo".to_string())
                        }
                        Error::BadTerminatedheader(_) => Err("BadTerminatedheader".to_string()),
                        Error::ExternalUnsupported => Err("ExternalUnsupported".to_string()),
                        Error::UnsupportedCompressionMethod(_) => {
                            Err("UnsupportedCompressionMethod".to_string())
                        }
                        Error::MaxMemLimited { .. } => Err("MaxMemLimited".to_string()),
                        Error::PasswordRequired => Err("PasswordRequired".to_string()),
                        Error::Unsupported(_) => Err("Unsupported".to_string()),
                        Error::MaybeBadPassword(_) => Err("MaybeBadPassword".to_string()),
                    });
                }
                match res {
                    Ok(_) => {
                        log::info!("{:?}: archive extracted!", name);
                        Ok(true)
                    }
                    Err(e) => {
                        log::error!("{:?}: failed to extract archive: {}", name, e);
                        Ok(false)
                    }
                }
            } else {
                Err("invalid file".to_string())
            };
        }
        return Err("invalid file path".to_string());
    }
}
