use std::collections::HashMap;
use std::time::Instant;

use bincode;
use clap::{Parser, Subcommand};
use indicatif::ProgressIterator;
use log::{info, warn};
use uom::si::f64::*;
use uom::si::length::light_year;
use uom::si::mass::kilogram;

mod shared;
use shared::astar;
use shared::data;
use shared::path;
use shared::raw;
use shared::tools;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the starmap from starmap.json
    Build {
        #[clap(short, long, default_value = "data/starmap.json")]
        source: String,
        #[clap(short, long, default_value = "data/starmap.bin")]
        output: String,
        #[clap(short = 'x', long, default_value = "200.0")]
        max_jump_distance: f64,
        #[clap(short = 'i', long, default_value = "0.0")]
        min_jump_distance: f64,
    },
    /// Find the shortest path between two stars
    Path {
        start_id: u32,
        end_id: u32,
        #[clap(short, long, default_value = "150")]
        jump_distance: u16,
        // #[clap(short, long, default_value = "fuel")]
        // optimize: eftb::calc::path::PathOptimize,
        // #[clap(short, long)]
        // use_smart_gates: bool,
        #[clap(short, long, default_value = "data/starmap.bin")]
        source: String,
    },
}

fn main() -> anyhow::Result<()> {
    use env_logger::Env;
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Build {
            source,
            output,
            max_jump_distance,
            min_jump_distance,
        }) => {
            info!("Ensuring output directory exists");
            if let Some(parent) = std::path::Path::new(output).parent() {
                std::fs::create_dir_all(parent)?;
            }
            info!("Loading raw data");
            let raw_star_data = raw::RawStarMap::from_file(source);
            let max_jump_dist: Length = Length::new::<light_year>(*max_jump_distance);
            let min_jump_dist: Length = Length::new::<light_year>(*min_jump_distance);
            info!("Building star map");
            let mut star_map: HashMap<data::SolarSystemId, data::Star> = HashMap::new();
            for (id_str, raw_star) in raw_star_data.solar_systems.iter() {
                let id = tools::system_id_to_u16(id_str.parse()?)?;
                let star = data::Star {
                    id,
                    x: raw_star.center[0],
                    y: raw_star.center[1],
                    z: raw_star.center[2],
                    connections: Vec::new(),
                };
                star_map.insert(id, star);
            }

            info!("Building connections from npc gates");
            let mut conn_count = 0;
            for raw_jump in raw_star_data.jumps.iter() {
                // rust only lets us borrow one mutable star at a time, so we can't add
                // from->to and to->from gates in the same block
                for (fid, tid) in [
                    (raw_jump.from_system_id, raw_jump.to_system_id),
                    (raw_jump.to_system_id, raw_jump.from_system_id),
                ] {
                    let fid = tools::system_id_to_u16(fid)?;
                    let tid = tools::system_id_to_u16(tid)?;
                    let to_star = star_map.get(&tid).unwrap().clone();
                    let from_star = star_map.get_mut(&fid).unwrap();
                    let distance: Length = from_star.distance(&to_star);
                    let conn_type = match raw_jump.jump_type {
                        0 => data::ConnType::NpcGate,
                        1 => data::ConnType::NpcGate, // What are these ???
                        _ => {
                            info!(
                                "{} -> {} is an unknown jump type ({})",
                                fid, tid, raw_jump.jump_type
                            );
                            continue;
                        }
                    };
                    from_star.connections.push(data::Connection {
                        id: conn_count,
                        conn_type,
                        distance: distance.get::<light_year>() as u16,
                        target: tid,
                    });
                    conn_count += 1;
                }
            }

            // info!("Building connections from smart gates");
            // let smart_gates: Vec<raw::RawSmartGate> =
            //     serde_json::from_str(&std::fs::read_to_string("data/smartgates.json")?)?;
            // for gate in smart_gates.iter() {
            //     if !star_map.contains_key(&gate.from) {
            //         warn!("Smart gate has unknown source {}", gate.from);
            //         continue;
            //     }
            //     if !star_map.contains_key(&gate.to) {
            //         warn!("Smart gate has unknown target {}", gate.to);
            //         continue;
            //     }
            //     let to_star = star_map.get(&gate.to).unwrap().clone();
            //     let from_star = star_map.get_mut(&gate.from).unwrap();
            //     let distance: Length = from_star.distance(&to_star);
            //     from_star.connections.push(data::Connection {
            //         id: conn_count,
            //         conn_type: data::ConnType::SmartGate,
            //         distance,
            //         target: gate.to,
            //     });
            //     conn_count += 1;
            // }

            info!("Building connections from jumps");            
            let cloned_star_map = star_map.clone();
            for star in star_map.values_mut().progress() {
                for other_star in cloned_star_map.values() {
                    if star.id == other_star.id {
                        continue;
                    }
                    let distance: Length = star.distance(&other_star);
                    if distance < max_jump_dist && distance > min_jump_dist {
                        star.connections.push(data::Connection {
                            id: conn_count,
                            conn_type: data::ConnType::Jump,
                            distance: distance.get::<light_year>() as u16,
                            target: other_star.id,
                        });
                        conn_count += 1;
                    }
                }
            }
            info!("Found {} connections", conn_count);

            info!("Sorting connections");
            // sort gates first, and then jumps by distance - then when we
            // reach a jump that is too long we can stop searching
            for star in star_map.values_mut().progress() {
                star.connections.sort_unstable();
            }

            info!("Saving star map");
            data::save_star_map(&star_map, output)?;
            info!("Complete");
        }
        Some(Commands::Path {
            start_id,
            end_id,
            jump_distance,
            // optimize,
            // use_smart_gates,
            source,
        }) => {
            info!("Loading star map");
            let now = Instant::now();
            let star_map = data::get_star_map(source)?;
            info!("Loaded star map in {:.3}", now.elapsed().as_secs_f64());

            let start = star_map
                .get(&tools::system_id_to_u16(*start_id).unwrap())
                .unwrap();
            let end = star_map
                .get(&tools::system_id_to_u16(*end_id).unwrap())
                .unwrap();

            info!("Finding path");
            let now = Instant::now();
            let path = path::calc_path(
                &star_map,
                start,
                end,
                *jump_distance,
                path::PathOptimize::Distance,
                false,
                Some(300),
            );
            info!("Found path in {:.3}", now.elapsed().as_secs_f64());
            match path {
                data::PathResult::Found((path, cost, stats)) => {
                    println!("Path from {} to {}:", start_id, end_id);
                    let mut last_id = start.id;
                    for conn in path {
                        println!(
                            "{} -> {} ({:?}, {} ly)",
                            tools::u16_to_system_id(last_id),
                            tools::u16_to_system_id(conn.target),
                            conn.conn_type,
                            conn.distance
                        );
                        last_id = conn.target;
                    }
                    println!(
                        "Visited: {} nodes, Cost: {}, Heuristic: {:?}, Successors: {:?}, Loop: {:?}, Time: {:?}",
                        stats.visited,
                        cost,
                        stats.heuristic_spend,
                        stats.successors_spend,
                        stats.loop_spend,
                        stats.total_time,
                    );
                }
                data::PathResult::NotFound(_) => {
                    warn!("No path found");
                }
                data::PathResult::Timeout(_) => {
                    warn!("Path search timed out");
                }
            }
        }
        None => {
            warn!("No command specified");
        }
    }

    Ok(())
}
