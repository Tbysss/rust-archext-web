mod extract;

use crate::extract::Extract;
use chrono::Local;
use env_logger::Builder;
use extract::Extractor;
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use glob::glob;
use log::LevelFilter;
use log::{debug, error, info};
use notify::EventKind;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::{io::Write, path::PathBuf};
use std::{path::Path, process::Command};

fn main() {
    let mut builder = Builder::new();
    builder.format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        });
    builder.filter_level(LevelFilter::Off);
    let env = env_logger::Env::new().filter_or("LOG_LEVEL", "info").write_style_or("LOG_STYLE", "always");
    builder.parse_env(env);
    builder.target(env_logger::Target::Stdout);
    builder.init();
    

    let path = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path (source)");
    let target_dir = std::env::args()
        .nth(2)
        .expect("Argument 2 needs to be a path (target)");
    let archive_dir = std::env::args()
        .nth(3)
        .expect("Argument 3 needs to be a path (archive)");
    let extractor = extract::Extractor::new(target_dir.as_str(), &archive_dir.as_str());
    info!("watching {}", path);
    info!("output to: {:?}", extractor.target_path);
    info!("archive to: {:?}", extractor.archive_dir);

    futures::executor::block_on(async {
        if let Err(e) = async_watch(path, &extractor).await {
            error!("error: {:?}", e)
        }
    });
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default()
            .with_poll_interval(std::time::Duration::from_secs(10))
            .with_compare_contents(true),
    )?;

    Ok((watcher, rx))
}

async fn async_watch<P: AsRef<Path>>(path: P, extractor: &Extractor) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(res) = rx.next().await {
        on_change(extractor, res)
    }

    Ok(())
}

fn on_change(extractor: &Extractor, res: Result<notify::Event>) {
    match res {
        Ok(event) => {
            debug!("event: {:?}", event);
            let file_path = &event.paths[0];
            if event.kind.is_modify() {
                if event.kind == EventKind::Modify(notify::event::ModifyKind::Name(notify::event::RenameMode::From))
                {                  
                    // zip inside folder?
                    let folder_path = file_path.parent().unwrap().to_str().expect("failed to turn path into str").to_owned();
                    let glob_pattern = folder_path + "/**/*.zip";
                    let mut new_zip = true;
                    while new_zip {
                        new_zip = false;
                        for entry in glob(&glob_pattern).expect("Failed to read glob pattern") {
                            match entry {
                                Ok(path) => {
                                    info!("found a zip inside folder: {:?}", path);
                                    
                                    let full_path = std::fs::canonicalize(&extractor.target_path).expect("failed to canonicalize path");
                                    let target_path = if path.starts_with(full_path) {
                                        // target is source -> reclaim structure
                                        path.parent()
                                    } else {
                                        None
                                    };
                                    // zip inside zip -> redo extract
                                    if extractor.extract(&path, target_path) {
                                        new_zip = true;
                                    } else {
                                        error!("invalid zip");
                                    }
                                    archive(extractor, &path);
                                }
                                Err(e) => println!("{:?}", e),
                            }
                        }
                    }
                }
            }
        }
        Err(e) => panic!("watch error: {:?}", e),
    }
}

fn archive(extractor: &Extractor, file_path: &PathBuf) {
    let mut archive_path = PathBuf::new();
    archive_path.push(&extractor.archive_dir);
    archive_path.push(file_path.file_name().unwrap());
    info!("{:?}: archive to {:?}", file_path, archive_path);
    Command::new("mv")
        .arg(file_path.as_os_str())
        .arg(archive_path.as_os_str())
        .status()
        .expect("failed to archive");
}
