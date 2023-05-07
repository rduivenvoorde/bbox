use actix_web::HttpRequest;
use core::fmt::Display;
use figment::providers::{Env, Format, Toml};
use figment::Figment;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::env;

/// Application configuration singleton
pub fn app_config() -> &'static Figment {
    static CONFIG: OnceCell<Figment> = OnceCell::new();
    &CONFIG.get_or_init(|| {
        Figment::new()
            .merge(Toml::file(
                env::var("BBOX_CONFIG").unwrap_or("bbox.toml".to_string()),
            ))
            .merge(Env::prefixed("BBOX_").split("__"))
    })
}

pub fn from_config_or_exit<'a, T: Default + Deserialize<'a>>(tag: &str) -> T {
    let config = app_config();
    match config.extract_inner(tag) {
        Ok(config) => config,
        Err(err) => {
            config_error_exit(err);
            Default::default()
        }
    }
}

pub fn from_config_opt_or_exit<'a, T: Deserialize<'a>>(tag: &str) -> Option<T> {
    let config = app_config();
    config
        .find_value(tag)
        .map(|_| config.extract_inner(tag).unwrap_or_else(error_exit))
        .ok()
}

pub fn config_error_exit<T: Display>(err: T) {
    eprintln!("Error during initialization: {err}");
    std::process::exit(1);
}

pub fn error_exit<T: Display, R>(err: T) -> R {
    eprintln!("Error during initialization: {err}");
    std::process::exit(1);
}

// -- Common configuration --

#[derive(Deserialize, Clone, Debug)]
pub struct WebserverCfg {
    #[serde(default = "default_server_addr")]
    pub server_addr: String,
    worker_threads: Option<usize>,
    public_server_url: Option<String>,
}

fn default_server_addr() -> String {
    "127.0.0.1:8080".to_string()
}

impl Default for WebserverCfg {
    fn default() -> Self {
        WebserverCfg {
            server_addr: default_server_addr(),
            worker_threads: None,
            public_server_url: None,
        }
    }
}

impl WebserverCfg {
    pub fn from_config() -> Self {
        from_config_or_exit("webserver")
    }
    pub fn worker_threads(&self) -> usize {
        self.worker_threads.unwrap_or(num_cpus::get())
    }
    pub fn public_server_url(&self, req: HttpRequest) -> String {
        if let Some(url) = &self.public_server_url {
            url.clone()
        } else {
            let conninfo = req.connection_info();
            format!("{}://{}", conninfo.scheme(), conninfo.host(),)
        }
    }
}

// -- Metrics --

#[derive(Deserialize, Default, Debug)]
pub struct MetricsCfg {
    pub prometheus: Option<PrometheusCfg>,
    pub jaeger: Option<JaegerCfg>,
}

#[derive(Deserialize, Debug)]
pub struct PrometheusCfg {
    pub path: String,
}

#[derive(Deserialize, Debug)]
pub struct JaegerCfg {
    pub agent_endpoint: String,
}

impl MetricsCfg {
    pub fn from_config() -> Self {
        from_config_or_exit("metrics")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use figment::providers::Env;
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    struct Package {
        name: String,
        edition: Option<String>,
    }

    #[test]
    fn toml_config() {
        let config = Figment::new()
            .merge(Toml::file("Cargo.toml"))
            .merge(Env::prefixed("CARGO_"));
        let package: Package = config.extract_inner("package").unwrap();
        assert_eq!(package.name, "bbox-common");
        assert_eq!(package.edition.unwrap(), "2018");
    }
}
