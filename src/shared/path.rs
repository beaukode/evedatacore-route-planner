use std::collections::HashMap;
use std::time::Duration;

use uom::si::f64::*;
use uom::si::length::light_year;

use log::info;

use super::data::*;
use super::astar::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    use_smart_gates: bool,
) -> Vec<(Connection, i64)> {
    let star = star_map.get(&conn.target).unwrap();
    star.connections
        .iter()
        // take gates and short jumps - stop searching after we
        // find a long jump
        .take_while(|c| c.conn_type != ConnType::Jump || c.distance <= jump_distance)
        // If we're not using smart gates, skip them
        .filter(|c| use_smart_gates || c.conn_type != ConnType::SmartGate)
        // Turn the connection into a (connection, cost) tuple
        .map(|c| {
            // info!("Successor: {} -> {} {} LY", star.id, c.target, c.distance);
            match (optimize, &c.conn_type) {
                // For shortest path, we only care about the distance
                // and don't care about the type of connection
                (PathOptimize::Distance, _) => (c.clone(), c.distance as i64),
                // For fuel efficient, we only care about the distance
                // if it's a jump
                (PathOptimize::Fuel, ConnType::Jump) => (c.clone(), c.distance as i64),
                // Gate connections are free (-ish. It still takes a tiny
                // amount of fuel to warp to a gate)
                (PathOptimize::Fuel, ConnType::NpcGate) => (c.clone(), 1),
                // Smart gates are slightly more expensive than NPC gates
                (PathOptimize::Fuel, ConnType::SmartGate) => (c.clone(), 2),
                // Treat all hops the same, we want to minimise the total
                (PathOptimize::Hops, _) => (c.clone(), 1),
            }
        })
        .collect()
}

/// Heuristic function for A* pathfinding
/// - Return an approximation of the cost from this connection to the end
/// - Must not return greater than the actual cost, or the path will be suboptimal
///   - Remember that in "optimise for fuel" mode, actual cost might be 1
pub fn heuristic(star_map: &HashMap<SolarSystemId, Star>, conn: &Connection, end: &Star) -> i64 {
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
    use_smart_gates: bool,
    timeout: Option<u64>,
) -> PathFindResult<Connection, i64> {
    let init_conn = Connection {
        id: 0,
        conn_type: ConnType::Jump,
        distance: 0,
        target: start.id,
    };
    let path = astar(
        &init_conn,
        |conn| successors(&star_map, conn, jump_distance, optimize, use_smart_gates),
        |conn| heuristic(&star_map, conn, end),
        |conn| conn.target == end.id,
        timeout,
    );

    match path {
        PathFindResult::Found((path, cost, stats)) => {
            // The first connection is the one we invented
            // to start the search, so we can skip it
            return PathFindResult::Found((path[1..].to_vec(), cost, stats));
        }
        PathFindResult::NotFound(stats) => {
            return PathFindResult::NotFound(stats)
        }
        PathFindResult::Timeout(stats) => {
            return PathFindResult::Timeout(stats)
        }
    }
}

