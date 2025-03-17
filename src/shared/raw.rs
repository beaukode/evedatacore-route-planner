use std::collections::HashMap;
use serde::{Deserialize, Deserializer};
use serde::de::{self, Visitor, SeqAccess};
use std::fmt;

// ====================================================================
// Data structures for the starmap pickle extracted from the client
// ====================================================================

#[derive(Debug, Deserialize)]
pub struct RawStarMap {
    pub jumps: Vec<RawJump>,
    #[serde(rename(deserialize = "solarSystems"))]
    pub solar_systems: HashMap<String, RawSolarSystem>,
}

impl RawStarMap {
    pub fn from_file(file: &str) -> Self {
        let file = std::fs::read_to_string(file).unwrap();
        serde_json::from_str(&file).unwrap()
    }
}

#[derive(Debug, Deserialize)]
pub struct RawJump {
    #[serde(rename(deserialize = "fromSystemID"))]
    pub from_system_id: u32,
    #[serde(rename(deserialize = "jumpType"))]
    pub jump_type: u8,
    #[serde(rename(deserialize = "toSystemID"))]
    pub to_system_id: u32,
}



#[derive(Debug, Deserialize)]
pub struct RawSolarSystem {
    pub center: [f64; 3],
}

#[derive(Debug, Deserialize)]
pub struct RawStar {
    #[serde(rename(deserialize = "solarSystemId"))]
    pub solar_system_id: u64,
    #[serde(rename(deserialize = "solarSystemName"))]
    pub solar_system_name: String,
}