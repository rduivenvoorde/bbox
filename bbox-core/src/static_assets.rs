use crate::static_files::EmbedFile;
use actix_web::{web, Error, HttpRequest};
use rust_embed::RustEmbed;
use std::path::PathBuf;

#[cfg(feature = "html")]
#[derive(RustEmbed)]
#[folder = "static/"]
struct Statics;

#[cfg(not(feature = "html"))]
type Statics = crate::static_files::EmptyDir;

async fn static_asset(req: HttpRequest) -> Result<EmbedFile, Error> {
    let filename = &req.path()[1..];
    // let filename = if filename == &PathBuf::from("") {
    //     PathBuf::from("index.html")
    // } else {
    //     filename.to_path_buf()
    // };
    Ok(EmbedFile::open::<Statics, _>(PathBuf::from(filename))?)
}

pub fn register_embedded_endpoints<E: RustEmbed>(cfg: &mut web::ServiceConfig) {
    let base_url = "/";
    for f in E::iter() {
        cfg.service(
            web::resource(&format!("{base_url}{}", &*f)).route(web::get().to(static_asset)),
        );
    }
}

pub fn register_endpoints(cfg: &mut web::ServiceConfig) {
    register_embedded_endpoints::<Statics>(cfg);
}
