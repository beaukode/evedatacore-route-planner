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
use utoipa::{OpenApi, ToSchema};

mod shared;
use shared::data;
use shared::data::{SolarSystemId, Star};
use shared::path;
use shared::search;
use shared::tools;

use crate::shared::api::{calc_near, calc_path};

#[rocket::get("/")]
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
