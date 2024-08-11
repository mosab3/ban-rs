// Responsible for watching the filesystem for changes

use std::borrow::BorrowMut;
use std::path::{self, PathBuf};
use std::io::{Seek, BufReader, BufRead};
use tokio::sync;
use tokio::{sync::mpsc, time::Duration};
use futures::future::join_all;
use notify::{Watcher, RecursiveMode, Result, RecommendedWatcher, Config};
use regex::Regex;
// use redis::
use crate::conf::{
    Apache2Config, NginxConfig, SshConfig, TomlConfig, CONFIG
};
use crate::helpers::calculate_optimal_channel_capacity;
use crate::banner::banner;

pub enum WatcherType {
    SHH,
    HTTP
}

pub enum HttpType {
    APACHE,
    NGINX
}

// fn get_file_path(watcher_type: WatcherType) -> path::PathBuf {
//     match watcher_type {
//         WatcherType::HTTP => {
//             get_http_logfile_path()
//         },
//         WatcherType::SHH => {
//             if is_red_hat_based() {
//                 return path::PathBuf::from("/var/log/secure");
//             } else {
//                 return path::PathBuf::from("/var/log/auth.log");
//             }
//         }
//     }
// }

// fn filesystem_watcher(path: &path::Path) {
//     assert!(path.is_file(), "Path is not a file");

// }

pub struct Record {
    pub ip: String,
    pub datetime: String,
    pub status_code: Option<u16>
}

// async fn file_read (path: path::PathBuf, regex: String, parent_tx: sync::mpsc::Sender<Record>) -> Result<()> {
async fn file_read (service_config: LogType, main_tx: sync::mpsc::Sender<Record>) -> Result<()> {
    let path = service_config.get_logpath();

    // get pos to end of file
    let mut f = std::fs::File::open(&path)?;
    let mut pos = std::fs::metadata(&path)?.len();

    // set up watcher
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    // watch
    for res in rx {
        match res {
            Ok(_event) => {
                // ignore any event that didn't change the pos
                if f.metadata()?.len() == pos {
                    continue;
                }

                // read from pos to end of file
                f.seek(std::io::SeekFrom::Start(pos + 1))?;

                // update post to end of file
                pos = f.metadata()?.len();

                let reader = BufReader::new(&f);
                for line in reader.lines() {
                    match service_config {
                        LogType::Ssh(ref config) => {
                            let re = Regex::new(&config.regex).unwrap();
                            if let Some(captures) = re.captures(&line.unwrap()) {
                                let ip = captures.name("ip").unwrap().as_str().to_string();
                                let datetime = captures.name("datetime").unwrap().as_str().to_string();
                                main_tx.send(Record {
                                    ip: ip,
                                    datetime: datetime,
                                    status_code: None,
                                });
                            }
                        },
                        LogType::Apache2(ref config) => {
                            let re = Regex::new(&config.regex).unwrap();
                            if let Some(captures) = re.captures(&line.unwrap()) {
                                let ip = captures.name("ip").unwrap().as_str().to_string();
                                let datetime = captures.name("datetime").unwrap().as_str().to_string();
                                let status_code: u16 = captures.name("status_code").unwrap().as_str().parse().unwrap();
                                main_tx.send(Record {
                                    ip: ip,
                                    datetime: datetime,
                                    status_code: Some(status_code),
                                });
                            }
                        },
                        LogType::Nginx(ref config) => {
                            let re = Regex::new(&config.regex).unwrap();
                            if let Some(captures) = re.captures(&line.unwrap()) {
                                let ip = captures.name("ip").unwrap().as_str().to_string();
                                let datetime = captures.name("datetime").unwrap().as_str().to_string();
                                let status_code: u16  = captures.name("status_code").unwrap().as_str().parse().unwrap();
                                main_tx.send(Record {
                                    ip: ip,
                                    datetime: datetime,
                                    status_code: Some(status_code),
                                });
                            }

                        }
                    }
                    // println!("> {:?}", line.unwrap());
                }
            }
            Err(error) => println!("{error:?}"),
        }
    }

    Ok(())

}

#[derive(Clone)]
enum LogType {
    Ssh(SshConfig),
    Apache2(Apache2Config),
    Nginx(NginxConfig)
}

impl LogType {
    fn get_logpath(&self) -> PathBuf {
        match self {
            LogType::Ssh(config) => config.logpath.clone(),
            LogType::Apache2(config) => config.logpath.clone(),
            LogType::Nginx(config) => config.logpath.clone()
        }
    }
}

#[test]
fn test_log_reader() {
    println!("")
}

fn record_data(record: Record) {
    todo!("register record data in redis on this conditions:
        1. if status code is 'failed authentication' status code
    ");
    todo!("check if the record is in the redis database");
    todo!("if the record in database and the ")
}

async fn log_reader(service_config: LogType, main_tx: sync::mpsc::Receiver<Record>) {
    let channel_capacity = calculate_optimal_channel_capacity();
    let redis_config = CONFIG.clone().redis;
    // let (parent_tx, mut parent_rx) = sync::mpsc::channel(channel_capacity);
    file_read(service_config.clone(), main_tx);
    let bantime = match service_config.clone() {
        LogType::Ssh(config) => config.bantime,
        LogType::Apache2(config) => config.bantime,
        LogType::Nginx(config) => config.bantime
    };
    // }


    // while let Some(message) =  parent_rx.recv().await {
    //     banner(message, bantime, _rx);
    // }

    // match service_config {
    //     LogType::Ssh(config) => {
    //         file_read(config.logpath, config.regex, parent_tx).await.unwrap();
    //     },
    //     LogType::Apache2(config) => {
    //         file_read(config.logpath, config.regex, parent_tx).await.unwrap();
    //     },
    //     LogType::Nginx(config) => {
    //         file_read(config.logpath, config.regex, parent_tx).await.unwrap();
    //     }
    // }
}

pub async fn watcher(main_tx: sync::mpsc::Sender<Record>) {
    let mut tasks = Vec::new();
    let local_config = CONFIG.clone();

    if CONFIG.ssh.enabled {
        tasks.push(log_reader(LogType::Ssh(local_config.ssh), main_tx));
    }
    if CONFIG.apache2.enabled {
        tasks.push(log_reader(LogType::Apache2(local_config.apache2), main_tx));
    }
    if CONFIG.nginx.enabled {
        tasks.push(log_reader(LogType::Nginx(local_config.nginx), main_tx));
    }

    tokio::task::spawn(join_all(tasks)).await.unwrap();
}