use std::path::{Path, PathBuf};
use std::fs;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize, Deserializer};
use crate::cli::arg_parser;
use crate::helpers::{is_red_hat_based, check_service_status};


static HOME_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let parent_dir = Path::new("/etc");
    parent_dir.join(env!("CARGO_PKG_NAME").to_lowercase())
});

static DEFAULT_CONFIG_DIR: Lazy<PathBuf> = Lazy::new(|| {
    HOME_DIR.join("config.toml")
});

const CONFIG_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let args = arg_parser();

    match args.config.clone() {
        Some(config) => {
            assert!(config.is_file(), "Path must be a valid config file");
            assert_eq!("toml", config.extension().unwrap(), "File must be a toml file");
            return config;
        }
        None => {
            DEFAULT_CONFIG_DIR.to_path_buf()
        }
    }
});

#[derive(Debug, PartialEq)]
enum ServiceType {
    SSH,
    APACHE2,
    NGINX
}

impl ServiceType {
    fn get_service(&self) -> String {
        match self {
            Self::APACHE2 => String::from("apache2"),
            Self::NGINX => String::from("nginx"),
            Self::SSH => String::from("sshd")
        }
    }
}

static VALIDATE_SERVICES: Lazy<Vec<ServiceType>> = Lazy::new(|| {
    let services: Vec<ServiceType> = vec![ServiceType::SSH, ServiceType::APACHE2, ServiceType::NGINX];
    let mut active_services = Vec::new();

    for service in services {
        if check_service_status(service.get_service().to_owned()) {
            active_services.push(service);
        }
    }

    active_services

});

// pub const CONFIG: TomlConfig = read_config();
pub static CONFIG: Lazy<TomlConfig> = Lazy::new(|| { read_config() });

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SshConfig {
    #[serde(default = "default_ssh_enabled")]
    pub enabled: bool,
    #[serde(default = "default_ssh_port")]
    port: u16,
    #[serde(default = "default_ssh_logpath")]
    pub logpath: PathBuf,
    #[serde(default = "default_ssh_regex")]
    pub regex: String,
    #[serde(default = "default_ssh_maxretry")]
    maxretry: u8,
    #[serde(default = "default_ssh_bantime")]
    pub bantime: u64,
    #[serde(default = "default_ssh_ignoreip")]
    ignoreip: Vec<Option<String>>,
}
fn default_ssh_enabled() -> bool { 
    if VALIDATE_SERVICES.contains(&ServiceType::SSH) {
        true
    } else {
        false
    }
 }
fn default_ssh_port() -> u16 { 22 }
fn default_ssh_logpath() -> PathBuf {
    if is_red_hat_based() {
        PathBuf::from("/var/log/secure")
    } else {
        PathBuf::from("/var/log/auth.log")
    }
}
fn default_ssh_regex() -> String { String::from(r"") }
fn default_ssh_maxretry() -> u8 { 3 }
fn default_ssh_bantime() -> u64 { 3600 }
fn default_ssh_ignoreip() -> Vec<Option<String>> { vec![] }

// Apache2 Struct
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Apache2Config {
    #[serde(default = "default_apache2_enabled")]
    pub enabled: bool,
    #[serde(default = "default_apache2_port")]
    port: Vec<u16>,
    #[serde(default = "default_apache2_logpath")]
    pub logpath: PathBuf,
    #[serde(default = "default_apache2_regex")]
    pub regex: String,
    #[serde(default = "default_apache2_maxretry")]
    maxretry: u8,
    #[serde(default = "default_apache2_bantime")]
    pub bantime: u64,
    #[serde(default = "default_apache2_ignoreip")]
    ignoreip: Vec<Option<String>>,
}
// Apache2 default values
fn default_apache2_enabled() -> bool {
    if VALIDATE_SERVICES.contains(&ServiceType::APACHE2) {
        true
    } else {
        false
    }
}
fn default_apache2_port() -> Vec<u16> { vec![80, 442] }
fn default_apache2_logpath() -> PathBuf {
    if is_red_hat_based() {
        PathBuf::from("/etc/httpd/logs/access_log")
    } else {
        PathBuf::from("var/log/apache/access.log")
    }
}
fn default_apache2_regex() -> String {
    String::from(r#"(?P<ip>\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}) - - \[(?P<datetime>[^\]]+)\] "[^"]+" (?P<status>\d{3})"#)
}
fn default_apache2_maxretry() -> u8 { 10 }
fn default_apache2_bantime() -> u64 { 600 }
fn default_apache2_ignoreip() -> Vec<Option<String>> { vec![] }

// Nginx Struct
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NginxConfig {
    #[serde(default = "default_nginx_enabled")]
    pub enabled: bool,
    #[serde(default = "default_nginx_port")]
    port: Vec<u16>,
    #[serde(default = "default_nginx_logpath")]
    pub logpath: PathBuf,
    #[serde(default = "default_nginx_regex")]
    pub regex: String,
    #[serde(default = "default_nginx_maxretry")]
    maxretry: u8,
    #[serde(default = "default_nginx_bantime")]
    pub bantime: u64,
    #[serde(default = "default_nginx_ignoreip")]
    ignoreip: Vec<Option<String>>,
}
// Nginx default values
fn default_nginx_enabled() -> bool {
    if VALIDATE_SERVICES.contains(&ServiceType::NGINX) {
        true
    } else {
        false
    }
}
fn default_nginx_port() -> Vec<u16> { vec![80, 442] }
fn default_nginx_logpath() -> PathBuf {
    PathBuf::from("/var/log/nginx/access")
}
fn default_nginx_regex() -> String {
    String::from(r#"(?P<ip>\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}) - - \[(?P<datetime>[^\]]+)\] "[^"]+" (?P<status>\d{3})"#)
}
fn default_nginx_maxretry() -> u8 { 10 }
fn default_nginx_bantime() -> u64 { 600 }
fn default_nginx_ignoreip() -> Vec<Option<String>> { vec![] }

// Redis Struct
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct RedisConfig {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_redis_host")]
    pub host: String,
    #[serde(default = "default_redis_port")]
    pub port: u16,
    #[serde(default = "default_redis_db")]
    pub db: i64,
}
// Redis default values
fn default_redis_host() -> String { String::from("127.0.0.1") }
fn default_redis_port() -> u16 { 6379 }
fn default_redis_db() -> i64 { 0 }


#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct TomlConfig {
    #[serde(default, deserialize_with = "deserialize_some_ssh")]
    pub ssh: SshConfig,
    #[serde(default, deserialize_with = "deserialize_some_apache2")]
    pub apache2: Apache2Config,
    #[serde(default, deserialize_with = "deserialize_some_nginx")]
    pub nginx: NginxConfig,
    #[serde(default, deserialize_with = "deserialize_some_redis")]
    pub redis: RedisConfig
}

fn deserialize_some_ssh<'de, D>(deserializer: D) -> Result<SshConfig, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::deserialize(deserializer)?.unwrap_or_default())
}

fn deserialize_some_apache2<'de, D>(deserializer: D) -> Result<Apache2Config, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::deserialize(deserializer)?.unwrap_or_default())
}

fn deserialize_some_nginx<'de, D>(deserializer: D) -> Result<NginxConfig, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::deserialize(deserializer)?.unwrap_or_default())
}

fn deserialize_some_redis<'de, D>(deserializer: D) -> Result<RedisConfig, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::deserialize(deserializer)?.unwrap_or_default())
}

impl Default for SshConfig {
    fn default() -> Self {
        SshConfig {
            enabled: default_ssh_enabled(),
            port: default_ssh_port(),
            logpath: default_ssh_logpath(),
            regex: default_ssh_regex(),
            maxretry: default_ssh_maxretry(),
            bantime: default_ssh_bantime(),
            ignoreip: default_ssh_ignoreip(),
        }
    }
}

impl Default for Apache2Config {
    fn default() -> Self {
        Apache2Config {
            enabled: default_apache2_enabled(),
            port: default_apache2_port(),
            logpath: default_apache2_logpath(),
            regex: default_apache2_regex(),
            maxretry: default_apache2_maxretry(),
            bantime: default_apache2_bantime(),
            ignoreip: default_apache2_ignoreip(),
        }
    }
}

impl Default for NginxConfig {
    fn default() -> Self {
        NginxConfig {
            enabled: default_nginx_enabled(),
            port: default_nginx_port(),
            logpath: default_nginx_logpath(),
            regex: default_nginx_regex(),
            maxretry: default_nginx_maxretry(),
            bantime: default_nginx_bantime(),
            ignoreip: default_nginx_ignoreip(),
        }
    }
}

trait Iterate {
    fn iter(&self) -> Box<dyn Iterator<Item = TomlConfig>>;
}

impl Iterate for TomlConfig {
    fn iter(&self) -> Box<dyn Iterator<Item = TomlConfig>> {
        Box::new(vec![TomlConfig::default()].into_iter())
    }
}

/// Creates the config directory and config file if it doesn't exist
fn make_dir_path() {
    if !HOME_DIR.is_dir() {
        let _ =fs::create_dir(HOME_DIR.as_path());
    }
    println!("{}", bool::from(DEFAULT_CONFIG_DIR.to_path_buf() == CONFIG_DIR.to_path_buf()));
    if !CONFIG_DIR.is_file() && DEFAULT_CONFIG_DIR.to_path_buf() == CONFIG_DIR.to_path_buf() {
        let toml_config = TomlConfig::default();
        let settings = toml::to_string(&toml_config).expect("Can not serialize config file");
        let config_file = fs::File::create(CONFIG_DIR.as_path());
        match config_file {
            Ok(_) => {
                let _ = fs::write(CONFIG_DIR.as_path(), settings);
            }
            Err(e) => {
                panic!("Can not create default config file: {:?}", e);
            }
        }
    }
}

#[test]
fn test_display_string() {
    println!("{}", ServiceType::SSH.get_service())
}

// fn validate_config() -> Vec<ServiceType> {

//     let services: Vec<ServiceType> = vec![ServiceType::SSH, ServiceType::APACHE2, ServiceType::NGINX, ServiceType::REDIS];
//     let mut active_services = Vec::new();

//     for service in services {
//         if check_service_status(service.get_service().to_owned()) {
//             active_services.push(service);
//         }
//     }

//     active_services

//     // assert!(active_services.contains(&ServiceType::SSH))

// }

// fn test_read_config() {

// }

pub fn read_config() -> TomlConfig {
    make_dir_path();

    let config_file = fs::read_to_string(CONFIG_DIR.as_path());

    match config_file {
        Ok(file) => {
            match toml::from_str::<TomlConfig>(&file) {
                Ok(config) => {

                    // Checklist:

                    // 1. Check if redis username and password are set
                    assert!(
                        config.redis.username != "" || config.redis.password != "",
                        "Redis username and password must be provided in the config file"
                    );

                    config
                }
                Err(e) => panic!("issue with a value in the config file: {:?}", e.message())
            }
        },
        Err(e) => panic!("Can Not Read Config File: {:?}", e)
    }

}
