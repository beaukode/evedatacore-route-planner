use log::{info, warn};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;

use rocket::fs::NamedFile;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use rocket::serde::json::Json;
use rocket::State;
use tokio::sync::Semaphore;

use serde::{Deserialize, Serialize};
use uom::si::f64::*;
use uom::si::length::light_year;
use uom::si::mass::kilogram;
use utoipa::{OpenApi, ToSchema};

use super::data;
use super::path;
use super::search;
use super::tools;

// ====================================================================
// common

#[derive(Debug, Deserialize, ToSchema)]
pub struct SmartGateLink {
    pub from: u32,
    pub to: u32,
    pub distance: u16,
    pub id: u32,
}

#[derive(Debug, Deserialize)]
pub struct EventPayload {
    pub from: u32,
    pub to: u32,
    pub jump_distance: u16,
    pub optimize: Option<data::PathOptimize>,
    pub smart_gates: Vec<SmartGateLink>,
}

//#[derive(Error)]
#[derive(Debug, Clone)]
pub struct CustomError(pub Status, pub String);

impl<'r> Responder<'r, 'static> for CustomError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .status(self.0)
            .header(ContentType::Text)
            .sized_body(self.1.len(), Cursor::new(self.1))
            .ok()
    }
}

impl From<anyhow::Error> for CustomError {
    fn from(err: anyhow::Error) -> Self {
        CustomError(Status::InternalServerError, format!("{:?}", err))
    }
}

// POST /api/path
#[derive(Debug, Deserialize, ToSchema)]
pub struct PathPayload {
    pub from: u32,
    pub to: u32,
    pub jump_distance: u16,
    pub optimize: Option<data::PathOptimize>,
    pub smart_gates: Vec<SmartGateLink>,
}

#[utoipa::path(
    post,
    path = "/path",
    responses(
        (status = 200, description = "Success", body = data::PathResult),
    ),
    request_body(content = PathPayload, description = "The payload to calculate the path"),
)]
#[rocket::post("/path", data = "<payload>")]
pub async fn calc_path(
    star_map: &State<data::StarMap>,
    semaphore: &State<Arc<Semaphore>>,
    payload: Json<PathPayload>,
) -> Json<data::PathResult> {
    info!("Payload: {:?}", payload);
    let _permit = semaphore
        .acquire()
        .await
        .expect("Max concurrent requests reached, try again later");

    let start_time = std::time::Instant::now();

    let mut smart_gates_map: data::SmartGatesMap = HashMap::new();
    for smart_gate in &payload.smart_gates {
        let from_id = tools::system_id_to_u16(smart_gate.from).unwrap();
        let to_id = tools::system_id_to_u16(smart_gate.to).unwrap();
        if !smart_gates_map.contains_key(&from_id) {
            smart_gates_map.insert(from_id, Vec::new());
        }
        smart_gates_map
            .get_mut(&from_id)
            .unwrap()
            .push(data::Connection {
                conn_type: data::ConnType::SmartGate,
                distance: smart_gate.distance,
                target: to_id,
                id: smart_gate.id,
            });
    }

    let elapsed = start_time.elapsed().as_millis();
    info!(
        "Time to create {} smart gates map: {}ms",
        payload.smart_gates.len(),
        elapsed
    );

    let start = star_map
        .get(&tools::system_id_to_u16(payload.from).unwrap())
        .unwrap();
    let end = star_map
        .get(&tools::system_id_to_u16(payload.to).unwrap())
        .unwrap();

    let path = path::calc_path(
        &star_map,
        &smart_gates_map,
        start,
        end,
        payload.jump_distance,
        payload.optimize.unwrap(),
        Some(25),
    );
    info!("Path: {:?}", path);

    Json(path)
}

// POST /api/near
#[derive(Debug, Deserialize, ToSchema)]
pub struct NearPayload {
    pub from: u32,
    pub distance: u16,
}

/// Find the nearest stars to a given star
///
/// Returns the nearest stars to a given star
#[utoipa::path(
    post,
    path = "/near",
    responses(
        (status = 200, description = "Success", body = data::NearResult),
    ),
    request_body(content = NearPayload, description = "The payload to calculate the nearest stars"),
)]
#[rocket::post("/near", data = "<payload>")]
pub fn calc_near(
    star_map: &State<data::StarMap>,
    payload: Json<NearPayload>,
) -> Json<data::NearResult> {
    info!("Payload: {:?}", payload);
    let start_time = std::time::Instant::now();

    let star = star_map
        .get(&tools::system_id_to_u16(payload.from).unwrap())
        .unwrap();
    let result = search::near(&star_map, star, payload.distance);
    Json(result)
}

#[derive(OpenApi)]
#[openapi(
    paths(calc_path, calc_near),
    components(schemas(data::PathResult, data::NearResult))
)]
pub struct ApiDoc;
