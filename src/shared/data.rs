use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uom::si::f64::*;
use uom::si::length::meter;

use log::info;

pub type ConnectionId = u32;
pub type SolarSystemId = u16;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ConnType {
    Gate,
    SmartGate,
    Jump,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Connection {
    pub id: ConnectionId,
    pub conn_type: ConnType,
    pub distance: u16,
    pub target: SolarSystemId,
}
impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Connection {}
impl std::hash::Hash for Connection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialOrd for Connection {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Connection {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.conn_type
            .cmp(&other.conn_type)
            .then_with(|| self.distance.partial_cmp(&other.distance).unwrap())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Star {
    pub id: SolarSystemId,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub connections: Vec<Connection>,
}

impl Star {
    pub fn distance(&self, other: &Star) -> Length {
        Length::new::<meter>(
            ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2))
                .sqrt(),
        )
    }
}
impl PartialEq for Star {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Star {}
impl std::hash::Hash for Star {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

pub fn save_star_map(star_map: &HashMap<SolarSystemId, Star>, dest: &str) -> anyhow::Result<()> {
    info!(
        "Saving star map to binary {}",
        bincode::serialized_size(&star_map)?
    );
    let file = std::fs::File::create(dest)?;
    bincode::serialize_into(file, &star_map)?;
    Ok(())
}

pub type StarMap = HashMap<SolarSystemId, Star>;

pub fn get_star_map(path: &str) -> anyhow::Result<StarMap> {
    let map: StarMap = bincode::deserialize(&std::fs::read(path)?)?;
    Ok(map)
}

#[derive(serde::Serialize, Debug)]
pub struct PathResultStats {
    pub cost: i64,
    pub total_time: u128,
    pub successors_spend: u128,
    pub loop_spend: u128,
    pub visited: u64,
}

#[derive(serde::Serialize, Debug)]
pub struct PathResultConnection {
    pub conn_type: ConnType,
    pub distance: u16,
    pub target: u32,
    pub id: u32,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PathResultStatus {
    Found,
    NotFound,
    Timeout,
}

#[derive(serde::Serialize, Debug)]
pub struct PathResult {
    pub status: PathResultStatus,
    pub path: Vec<PathResultConnection>,
    pub stats: PathResultStats,
}

#[derive(serde::Serialize, Debug)]
pub struct NearResult {
    pub connections: Vec<PathResultConnection>,
}
