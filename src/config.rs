use std::fs::File;
use std::sync::Arc;

use num_cpus;
use serde::Deserialize;
use serde_yaml;

use crate::models::{ChannelType, ServiceId};

pub fn load(config_path: &str) -> Arc<Config> {
    let reader = File::open(config_path)
        .unwrap_or_else(|err| {
            panic!("Failed to open {}: {}", config_path, err);
        });
    let config = serde_yaml::from_reader(reader)
        .unwrap_or_else(|err| {
            panic!("Failed to paser {}: {}", config_path, err);
        });
    Arc::new(config)
}

// result

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(default)]
    pub epg: EpgConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub channels: Vec<ChannelConfig>,
    #[serde(default)]
    pub tuners: Vec<TunerConfig>,
    #[serde(default)]
    pub filters: FiltersConfig,
    #[serde(default)]
    pub jobs: JobsConfig,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct EpgConfig {
    #[serde(default)]
    pub cache_dir: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct ServerConfig {
    #[serde(default = "default_server_address")]
    pub address: String,
    #[serde(default = "default_server_port")]
    pub port: u16,
    #[serde(default = "default_server_workers")]
    pub workers: usize,
}

fn default_server_address() -> String { "localhost".to_string() }
fn default_server_port() -> u16 { 40772 }
fn default_server_workers() -> usize { num_cpus::get() }

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            address: default_server_address(),
            port: default_server_port(),
            workers: default_server_workers(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct ChannelConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub channel_type: ChannelType,
    pub channel: String,
    #[serde(default)]
    pub services: Vec<ServiceId>,
    #[serde(default)]
    pub excluded_services: Vec<ServiceId>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct TunerConfig {
    pub name: String,
    #[serde(rename = "types")]
    pub channel_types: Vec<ChannelType>,
    pub command: String,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct FiltersConfig {
    #[serde(default)]
    pub pre_filter: String,
    #[serde(default = "default_service_filter")]
    pub service_filter: String,
    #[serde(default = "default_program_filter")]
    pub program_filter: String,
    #[serde(default)]
    pub post_filter: String,
}

impl Default for FiltersConfig {
    fn default() -> Self {
        FiltersConfig {
            pre_filter: String::new(),
            service_filter: default_service_filter(),
            program_filter: default_program_filter(),
            post_filter: String::new(),
        }
    }
}

fn default_service_filter() -> String {
    "mirakc-arib filter-service --sid={{sid}}".to_string()
}

fn default_program_filter() -> String {
    // The --pre-streaming option is used in order to avoid the issue#1313 in
    // actix/actix-web.  PSI/SI TS packets will be sent before the program
    // starts.
    //
    // See masnagam/rust-case-studies for details.
    "mirakc-arib filter-program --sid={{sid}} --eid={{eid}} \
     --clock-pcr={{clock_pcr}} --clock-time={{clock_time}} \
     --start-margin=5000 --end-margin=5000 --pre-streaming".to_string()
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct JobsConfig {
    #[serde(default = "default_scan_services_job")]
    pub scan_services: JobConfig,
    #[serde(default = "default_sync_clocks_job")]
    pub sync_clocks: JobConfig,
    #[serde(default = "default_update_schedules_job")]
    pub update_schedules: JobConfig,
}

impl Default for JobsConfig {
    fn default() -> Self {
        JobsConfig {
            scan_services: default_scan_services_job(),
            sync_clocks: default_sync_clocks_job(),
            update_schedules: default_update_schedules_job(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct JobConfig {
    pub command: String,
    pub schedule: String,
}

fn default_scan_services_job() -> JobConfig {
    JobConfig {
        command: "mirakc-arib scan-services".to_string(),
        schedule: "0 31 5 * * * *".to_string(),
    }
}

fn default_sync_clocks_job() -> JobConfig {
    JobConfig {
        command: "mirakc-arib sync-clocks".to_string(),
        schedule: "0 3 12 * * * *".to_string(),
    }
}

fn default_update_schedules_job() -> JobConfig {
    JobConfig {
        command: "mirakc-arib collect-eits".to_string(),
        schedule: "0 7,37 * * * * *".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let result = serde_yaml::from_str::<Config>("{}");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Default::default());

        let result = serde_yaml::from_str::<Config>(r#"
            epg:
              cache-dir: /path/to/epg
        "#);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Config {
            epg: EpgConfig {
                cache_dir: Some("/path/to/epg".to_string()),
            },
            server: Default::default(),
            channels: vec![],
            tuners: vec![],
            jobs: Default::default(),
            filters: Default::default(),
        });

        let result = serde_yaml::from_str::<Config>(r#"
            unknown:
              property: value
        "#);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Default::default());
    }

    #[test]
    fn test_epg_config() {
    }

    #[test]
    fn test_server_config() {
        assert_eq!(
            serde_yaml::from_str::<ServerConfig>("{}").unwrap(),
            Default::default());

        assert_eq!(
            serde_yaml::from_str::<ServerConfig>(r#"
                address: '0.0.0.0'
            "#).unwrap(),
            ServerConfig {
                address: "0.0.0.0".to_string(),
                port: default_server_port(),
                workers: default_server_workers(),
            });

        assert_eq!(
            serde_yaml::from_str::<ServerConfig>(r#"
                port: 11111
            "#).unwrap(),
            ServerConfig {
                address: default_server_address(),
                port: 11111,
                workers: default_server_workers(),
            });

        assert_eq!(
            serde_yaml::from_str::<ServerConfig>(r#"
                workers: 2
            "#).unwrap(),
            ServerConfig {
                address: default_server_address(),
                port: default_server_port(),
                workers: 2,
            });
    }

    #[test]
    fn test_channel_config() {
        assert!(serde_yaml::from_str::<ChannelConfig>("{}").is_err());

        assert_eq!(
            serde_yaml::from_str::<ChannelConfig>(r#"
                name: x
                type: GR
                channel: y
            "#).unwrap(),
            ChannelConfig {
                name: "x".to_string(),
                channel_type: ChannelType::GR,
                channel: "y".to_string(),
                services: vec![],
                excluded_services: vec![],
                disabled: false,
            });

        assert_eq!(
            serde_yaml::from_str::<ChannelConfig>(r#"
                name: x
                type: GR
                channel: y
                disabled: true
            "#).unwrap(),
            ChannelConfig {
                name: "x".to_string(),
                channel_type: ChannelType::GR,
                channel: "y".to_string(),
                services: vec![],
                excluded_services: vec![],
                disabled: true,
            });

        assert_eq!(
            serde_yaml::from_str::<ChannelConfig>(r#"
                name: x
                type: GR
                channel: y
                excluded-services: [100]
            "#).unwrap(),
            ChannelConfig {
                name: "x".to_string(),
                channel_type: ChannelType::GR,
                channel: "y".to_string(),
                services: vec![],
                excluded_services: vec![100.into()],
                disabled: false,
            });

        assert!(
            serde_yaml::from_str::<ChannelConfig>(r#"
                name: x
                type: WOWOW
                channel: y
            "#).is_err());
    }

    #[test]
    fn test_tuner_config() {
        assert!(serde_yaml::from_str::<TunerConfig>("{}").is_err());

        assert_eq!(
            serde_yaml::from_str::<TunerConfig>(r#"
                name: x
                types: [GR, BS, CS, SKY]
                command: open tuner
            "#).unwrap(),
            TunerConfig {
                name: "x".to_string(),
                channel_types: vec![ChannelType::GR,
                                    ChannelType::BS,
                                    ChannelType::CS,
                                    ChannelType::SKY],
                command: "open tuner".to_string(),
                disabled: false,
            });

        assert_eq!(
            serde_yaml::from_str::<TunerConfig>(r#"
                name: x
                types: [GR, BS, CS, SKY]
                command: open tuner
                disabled: true
            "#).unwrap(),
            TunerConfig {
                name: "x".to_string(),
                channel_types: vec![ChannelType::GR,
                                    ChannelType::BS,
                                    ChannelType::CS,
                                    ChannelType::SKY],
                command: "open tuner".to_string(),
                disabled: true,
            });

        assert!(
            serde_yaml::from_str::<TunerConfig>(r#"
                name: x
                types: [WOWOW]
                command: open tuner
            "#).is_err());
    }

    #[test]
    fn test_filters_config() {
        assert_eq!(
            serde_yaml::from_str::<FiltersConfig>("{}").unwrap(),
            Default::default());

        assert_eq!(
            serde_yaml::from_str::<FiltersConfig>(r#"
                pre-filter: filter
            "#).unwrap(),
            FiltersConfig {
                pre_filter: "filter".to_string(),
                service_filter: default_service_filter(),
                program_filter: default_program_filter(),
                post_filter: String::new(),
            });

        assert_eq!(
            serde_yaml::from_str::<FiltersConfig>(r#"
                service-filter: filter
            "#).unwrap(),
            FiltersConfig {
                pre_filter: String::new(),
                service_filter: "filter".to_string(),
                program_filter: default_program_filter(),
                post_filter: String::new(),
            });

        assert_eq!(
            serde_yaml::from_str::<FiltersConfig>(r#"
                program-filter: filter
            "#).unwrap(),
            FiltersConfig {
                pre_filter: String::new(),
                service_filter: default_service_filter(),
                program_filter: "filter".to_string(),
                post_filter: String::new(),
            });

        assert_eq!(
            serde_yaml::from_str::<FiltersConfig>(r#"
                post-filter: filter
            "#).unwrap(),
            FiltersConfig {
                pre_filter: String::new(),
                service_filter: default_service_filter(),
                program_filter: default_program_filter(),
                post_filter: "filter".to_string(),
            });
    }

    #[test]
    fn test_jobs_config() {
        assert_eq!(
            serde_yaml::from_str::<JobsConfig>("{}").unwrap(),
            Default::default());

        assert_eq!(
            serde_yaml::from_str::<JobsConfig>(r#"
                scan-services:
                  command: job
                  schedule: '*'
            "#).unwrap(),
            JobsConfig {
                scan_services: JobConfig {
                    command: "job".to_string(),
                    schedule: "*".to_string(),
                },
                sync_clocks: default_sync_clocks_job(),
                update_schedules: default_update_schedules_job(),
            });

        assert_eq!(
            serde_yaml::from_str::<JobsConfig>(r#"
                sync-clocks:
                  command: job
                  schedule: '*'
            "#).unwrap(),
            JobsConfig {
                scan_services: default_scan_services_job(),
                sync_clocks: JobConfig {
                    command: "job".to_string(),
                    schedule: "*".to_string(),
                },
                update_schedules: default_update_schedules_job(),
            });

        assert_eq!(
            serde_yaml::from_str::<JobsConfig>(r#"
                update-schedules:
                  command: job
                  schedule: '*'
            "#).unwrap(),
            JobsConfig {
                scan_services: default_scan_services_job(),
                sync_clocks: default_sync_clocks_job(),
                update_schedules: JobConfig {
                    command: "job".to_string(),
                    schedule: "*".to_string(),
                },
            });
    }

    #[test]
    fn test_job_config() {
        assert!(serde_yaml::from_str::<JobConfig>("{}").is_err());
        assert!(
            serde_yaml::from_str::<JobConfig>(r#"{"command":""}"#).is_err());
        assert!(
            serde_yaml::from_str::<JobConfig>(r#"{"schedule":""}"#).is_err());
    }
}
