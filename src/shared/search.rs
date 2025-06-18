use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use uom::si::f64::*;
use uom::si::length::light_year;

use log::info;

use super::astar;
use super::data::*;
use super::tools;

pub fn near(star_map: &HashMap<SolarSystemId, Star>, star: &Star, distance: u16) -> NearResult {
    NearResult {
        connections: star
            .connections
            .iter()
            .take_while(|c| c.distance <= distance)
            .filter(|c| c.conn_type == ConnType::Jump)
            .map(|c| PathResultConnection {
                conn_type: c.conn_type.clone(),
                distance: c.distance,
                target: tools::u16_to_system_id(c.target),
                id: c.id,
            })
            .collect(),
    }
}
