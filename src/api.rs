#[macro_use]
extern crate rocket;

use log::{info, warn};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;

use rocket::fs::NamedFile;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use uom::si::f64::*;
use uom::si::length::light_year;
use uom::si::mass::kilogram;

mod shared;
use shared::data;
use shared::data::{SolarSystemId, Star};
use shared::path;
use shared::search;
use shared::tools;

// ====================================================================
// common

#[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
struct PathPayload {
    pub from: u32,
    pub to: u32,
    pub jump_distance: u16,
    pub optimize: Option<data::PathOptimize>,
    pub smart_gates: Vec<SmartGateLink>,
}

#[post("/path", data = "<payload>")]
fn calc_path(
    star_map: &State<data::StarMap>,
    payload: Json<PathPayload>,
) -> Json<data::PathResult> {
    info!("Payload: {:?}", payload);
    let start_time = std::time::Instant::now();

    let mut star_map_copy = star_map.inner().clone();
    for smart_gate in &payload.smart_gates {
        let from_id = tools::system_id_to_u16(smart_gate.from).unwrap();
        let to_id = tools::system_id_to_u16(smart_gate.to).unwrap();
        if let Some(from_system) = star_map_copy.get_mut(&from_id) {
            from_system.connections.insert(
                0,
                data::Connection {
                    conn_type: data::ConnType::SmartGate,
                    distance: smart_gate.distance,
                    target: to_id,
                    id: smart_gate.id,
                },
            );
        }
    }

    let elapsed = start_time.elapsed().as_millis();
    info!(
        "Time to inject {} smart gates: {}ms",
        payload.smart_gates.len(),
        elapsed
    );

    let start = star_map_copy
        .get(&tools::system_id_to_u16(payload.from).unwrap())
        .unwrap();
    let end = star_map_copy
        .get(&tools::system_id_to_u16(payload.to).unwrap())
        .unwrap();

    let path = path::calc_path(
        &star_map_copy,
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
#[derive(Debug, Deserialize)]
struct NearPayload {
    pub from: u32,
    pub max_distance: u16,
}

#[post("/near", data = "<payload>")]
fn calc_near(
    star_map: &State<data::StarMap>,
    payload: Json<NearPayload>,
) -> Json<data::NearResult> {
    info!("Payload: {:?}", payload);
    let start_time = std::time::Instant::now();

    let star = star_map
        .get(&tools::system_id_to_u16(payload.from).unwrap())
        .unwrap();
    let result = search::near(&star_map, star, payload.max_distance);
    Json(result)
}

#[get("/")]
fn root() -> &'static str {
    ""
}

#[launch]
fn rocket() -> _ {
    use env_logger::Env;
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let path = std::env::var("STARMAP_PATH").unwrap_or_else(|_| String::from("data/starmap.bin"));
    info!("Loading star map from {}", path);
    let start = std::time::Instant::now();
    let map: data::StarMap = data::get_star_map(&path).unwrap();

    info!(
        "Star map loaded with {} stars in {}ms",
        map.len(),
        start.elapsed().as_millis()
    );

    rocket::build()
        .manage(map)
        .mount("/api", routes![calc_path, calc_near])
        .mount("/", routes![root])
}
