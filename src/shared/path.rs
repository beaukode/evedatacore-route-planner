use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use uom::si::f64::*;
use uom::si::length::light_year;

use log::info;

use super::astar;
use super::data::*;
use super::tools;

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PathOptimize {
    Fuel,
    Distance,
    Hops,
}

/// Given a connection, return a list of all possible next-connections,
/// and what each of those connections costs
fn successors(
    star_map: &HashMap<SolarSystemId, Star>,
    conn: &Connection,
    jump_distance: u16,
    optimize: PathOptimize,
) -> Vec<(Connection, i64)> {
    let star = star_map.get(&conn.target).unwrap();
    star.connections
        .iter()
        // take gates and short jumps - stop searching after we
        // find a long jump
        .take_while(|c| c.conn_type != ConnType::Jump || c.distance <= jump_distance)
        // Turn the connection into a (connection, cost) tuple
        .map(|c| {
            match (optimize, &c.conn_type) {
                // For shortest path, we only care about the distance
                // and don't care about the type of connection
                (PathOptimize::Distance, _) => (c.clone(), c.distance as i64),
                // For fuel efficient, we penalise jumps
                (PathOptimize::Fuel, ConnType::Jump) => (c.clone(), c.distance as i64),
                // Over gates, we only count half the distance
                (PathOptimize::Fuel, ConnType::Gate) => (c.clone(), 1),
                (PathOptimize::Fuel, ConnType::SmartGate) => (c.clone(), 1),
                // Treat all hops the same, we want to minimise the total
                (PathOptimize::Hops, _) => (c.clone(), 100),
            }
        })
        .collect()
}

/// Heuristic function for A* pathfinding
/// - Return an approximation of the cost from this connection to the end
/// - Must not return greater than the actual cost, or the path will be suboptimal
///   - Remember that in "optimise for fuel" mode, actual cost might be 1
pub fn heuristic(
    star_map: &HashMap<SolarSystemId, Star>,
    conn: &Connection,
    end: &Star,
    optimize: PathOptimize,
) -> i64 {
    if conn.conn_type != ConnType::Jump && optimize == PathOptimize::Fuel {
        return 0;
    }
    let d = star_map
        .get(&conn.target)
        .unwrap()
        .distance(end)
        .get::<light_year>();
    return d as i64;
}

pub fn calc_path(
    star_map: &HashMap<SolarSystemId, Star>,
    start: &Star,
    end: &Star,
    jump_distance: u16,
    optimize: PathOptimize,
    timeout: Option<u64>,
) -> PathResult {
    let init_conn = Connection {
        id: 0,
        conn_type: ConnType::Jump,
        distance: 0,
        target: start.id,
    };
    let path = astar::astar(
        &init_conn,
        |conn| successors(&star_map, conn, jump_distance, optimize),
        |conn| heuristic(&star_map, conn, end, optimize),
        |conn| conn.target == end.id,
        timeout,
    );

    match path {
        astar::PathFindResult::Found((path, cost, stats)) => {
            // The first connection is the one we invented
            // to start the search, so we can skip it
            let path = path[1..]
                .to_vec()
                .iter()
                .map(|c| PathResultConnection {
                    conn_type: c.conn_type.clone(),
                    distance: c.distance,
                    target: tools::u16_to_system_id(c.target),
                    id: c.id,
                })
                .collect();
            return PathResult {
                status: PathResultStatus::Found,
                path,
                stats: PathResultStats {
                    cost,
                    total_time: stats.total_time.as_millis(),
                    successors_spend: stats.successors_spend.as_millis(),
                    loop_spend: stats.loop_spend.as_millis(),
                    visited: stats.visited,
                },
            };
        }
        astar::PathFindResult::NotFound(stats) => {
            return PathResult {
                status: PathResultStatus::NotFound,
                path: vec![],
                stats: PathResultStats {
                    cost: 0,
                    total_time: stats.total_time.as_millis(),
                    successors_spend: stats.successors_spend.as_millis(),
                    loop_spend: stats.loop_spend.as_millis(),
                    visited: stats.visited,
                },
            };
        }
        astar::PathFindResult::Timeout(stats) => {
            return PathResult {
                status: PathResultStatus::Timeout,
                path: vec![],
                stats: PathResultStats {
                    cost: 0,
                    total_time: stats.total_time.as_millis(),
                    successors_spend: stats.successors_spend.as_millis(),
                    loop_spend: stats.loop_spend.as_millis(),
                    visited: stats.visited,
                },
            };
        }
    }
}
