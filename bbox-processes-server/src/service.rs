use crate::config::ProcessesServerCfg;
use crate::dagster::DagsterBackend;
use actix_web::web;
use async_trait::async_trait;
use bbox_common::ogcapi::ApiLink;
use bbox_common::service::{CoreService, OgcApiService};

#[derive(Clone)]
pub struct ProcessesService {
    pub backend: Option<DagsterBackend>,
}

#[async_trait]
impl OgcApiService for ProcessesService {
    async fn from_config() -> Self {
        let config = ProcessesServerCfg::from_config();
        // if !config.has_backend() {
        //     info!("Missing processing backend configuration - skipping endpoints");
        //     return;
        // }
        let backend = config.dagster_backend.map(|_cfg| DagsterBackend::new());
        ProcessesService { backend }
    }
    fn conformance_classes(&self) -> Vec<String> {
        vec![
            "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/core".to_string(),
            "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/json".to_string(),
            // |Core|http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/core|
            // |OGC Process Description|http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/ogc-process-description|
            // |JSON|http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/json|
            // |HTML|http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/html|
            // |OpenAPI Specification 3.0|http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/oas30|
            // |Job list|http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/job-list|
            // |Callback|http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/callback|
            // |Dismiss|http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/dismiss|
            "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/oas30".to_string(),
        ]
    }
    fn landing_page_links(&self, _api_base: &str) -> Vec<ApiLink> {
        vec![ApiLink {
            href: "/processes".to_string(),
            rel: Some("processes".to_string()),
            type_: Some("application/json".to_string()),
            title: Some("OGC API processes list".to_string()),
            hreflang: None,
            length: None,
        }]
    }
    fn openapi_yaml(&self) -> Option<&str> {
        Some(include_str!("openapi.yaml"))
    }
    fn register_endpoints(&self, cfg: &mut web::ServiceConfig, core: &CoreService) {
        self.register(cfg, core)
    }
}