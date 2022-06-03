//! Endpoints according to <https://ogcapi.ogc.org/processes/> API

use crate::dagster;
use crate::models::*;
use actix_web::{http::StatusCode, web, HttpRequest, HttpResponse};
use bbox_common::api::{ExtendApiDoc, OgcApiInventory};
use bbox_common::ogcapi::ApiLink;
use log::{info, warn};
use utoipa::OpenApi;

/// retrieve the list of available processes
///
/// The list of processes contains a summary of each process the OGC API -
/// Processes offers, including the link to a more detailed description
/// of the process.
#[utoipa::path(
    get,
    path = "/processes",
    operation_id = "ProcessList",
    tag = "processes",
    responses(
        (status = 200, body = ProcessList),
    ),
)]
async fn processes(_req: HttpRequest) -> HttpResponse {
    let jobs = dagster::query_jobs().await.unwrap_or_else(|e| {
        warn!("Dagster backend error: {e}");
        Vec::new()
    });
    let processes = jobs
        .iter()
        .map(|job| {
            let mut process = ProcessSummary::new(job.name.clone(), "1.0.0".to_string());
            process.description = job.description.clone();
            process
        })
        .collect::<Vec<_>>();
    /* Example:
    {
      "processes": [
        {
          "id": "EchoProcess",
          "title": "EchoProcess",
          "version": "1.0.0",
          "jobControlOptions": [
            "async-execute",
            "sync-execute"
          ],
          "outputTransmission": [
            "value",
            "reference"
          ],
          "additionalParameters": {
            "title": "string",
            "role": "string",
            "href": "string",
            "parameters": [
              {
                "name": "string",
                "value": [
                  "string",
                  0,
                  0,
                  [
                    null
                  ],
                  {}
                ]
              }
            ]
          },
          "links": [
            {
              "href": "https://processing.example.org/oapi-p/processes/EchoProcess",
              "type": "application/json",
              "rel": "self",
              "title": "process description"
            }
          ]
        }
      ],
      "links": [
        {
          "href": "https://processing.example.org/oapi-p/processes?f=json",
          "rel": "self",
          "type": "application/json"
        },
        {
          "href": "https://processing.example.org/oapi-p/processes?f=html",
          "rel": "alternate",
          "type": "text/html"
        }
      ]
    }
    */
    let resp = ProcessList {
        processes,
        links: Vec::new(),
    };

    HttpResponse::Ok().json(resp)
}

/// execute a process
///
/// Create a new job.
// For more information, see [Section 7.11](https://docs.ogc.org/is/18-062/18-062.html#sc_create_job).
#[utoipa::path(
    post,
    path = "/processes/{processID}/execution",
    operation_id = "execute",
    tag = "Execute",
    responses(
        (status = 200),
    ),
)]
async fn execute(
    process_id: web::Path<String>,
    parameters: web::Json<dagster::Execute>,
) -> HttpResponse {
    info!("Process parameters: {parameters:?}");
    match dagster::execute_job(&process_id, &*parameters).await {
        /* responses:
                200:
                  $ref: 'http://schemas.opengis.net/ogcapi/processes/part1/1.0/openapi/responses/ExecuteSync.yaml'
                201:
                  $ref: "http://schemas.opengis.net/ogcapi/processes/part1/1.0/openapi/responses/ExecuteAsync.yaml"
                404:
                  $ref: "http://schemas.opengis.net/ogcapi/processes/part1/1.0/openapi/responses/NotFound.yaml"
                500:
                  $ref: "http://schemas.opengis.net/ogcapi/processes/part1/1.0/openapi/responses/ServerError.yaml"
        */
        Ok(job_id) => HttpResponse::build(StatusCode::CREATED).json(job_id), // TODO: type ExecuteAsync
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()), // TODO: type ServerError
    }
}

#[derive(OpenApi)]
#[openapi(handlers(processes, execute), components(ProcessList))]
pub struct ApiDoc;

pub fn init_service(api: &mut OgcApiInventory, openapi: &mut utoipa::openapi::OpenApi) {
    api.landing_page_links.push(ApiLink {
        href: "/processes".to_string(),
        rel: Some("processes".to_string()),
        type_: Some("application/json".to_string()),
        title: Some("OGC API processes list".to_string()),
        hreflang: None,
        length: None,
    });
    api.conformance_classes.extend(vec![
        "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/core".to_string(),
        "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/ogc-process-description"
            .to_string(),
        "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/json".to_string(),
        "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/oas30".to_string(),
    ]);
    openapi.extend(ApiDoc::openapi());
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/processes").route(web::get().to(processes)))
        .service(web::resource("/processes/{processID}/execution").route(web::post().to(execute)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{body, dev::Service, http, test, App, Error};

    #[actix_web::test]
    async fn test_process_list() -> Result<(), Error> {
        let app = test::init_service(
            App::new().service(web::resource("/processes").route(web::get().to(processes))),
        )
        .await;

        let req = test::TestRequest::get().uri("/processes").to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = body::to_bytes(resp.into_body()).await?;

        assert!(response_body.starts_with(b"{\"links\":["));

        Ok(())
    }
}
