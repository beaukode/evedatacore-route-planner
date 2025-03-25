use lambda_runtime::{tracing, Error, LambdaEvent};
use serde::{Deserialize, Deserializer};

use crate::shared::astar;
use crate::shared::data;
use crate::shared::path;
use crate::shared::tools;

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
    pub optimize: Option<path::PathOptimize>,
    pub smart_gates: Vec<SmartGateLink>,
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
pub(crate) async fn function_handler(
    event: LambdaEvent<EventPayload>,
    star_map: &data::StarMap,
) -> Result<data::PathResult, Error> {
    // Extract some useful information from the request
    let payload = event.payload;
    tracing::info!("Payload: {:?}", payload);

    let start_time = std::time::Instant::now();

    let mut star_map_copy = star_map.clone();    
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
    tracing::info!(
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
    tracing::info!("Path: {:?}", path);
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_runtime::{Context, LambdaEvent};

    #[tokio::test]
    async fn test_event_handler() {
        let path =
            std::env::var("STARMAP_PATH").unwrap_or_else(|_| String::from("data/starmap.bin"));

        if !std::path::Path::new(&path).exists() {
            panic!(
                "Star map file not found at {}, run: cargo run --release build",
                path
            );
        }
        println!("Loading star map... {}", path);
        let map: data::StarMap = data::get_star_map(&path).unwrap();
        let map_ref = &map;

        let event = LambdaEvent::new(
            EventPayload {
                from: 30001573,
                to: 30013956,
                jump_distance: 150,
                optimize: Some(path::PathOptimize::Fuel),
                smart_gates: vec![],
            },
            Context::default(),
        );
        let response = function_handler(event, map_ref).await.unwrap();
        assert!(
            matches!(response.status, data::PathResultStatus::Found),
            "Expected to find a path"
        );
    }
}
