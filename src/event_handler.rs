use lambda_runtime::{tracing, Error, LambdaEvent};
use serde::{Deserialize, Deserializer};

use crate::shared::astar;
use crate::shared::data;
use crate::shared::path;
use crate::shared::tools;

#[derive(Debug, Deserialize)]
pub struct EventPayload {
    pub from: u32,
    pub to: u32,
    pub jump_distance: u16,
    pub optimize: Option<path::PathOptimize>,
    pub use_smart_gates: bool,
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

    let start = star_map
        .get(&tools::system_id_to_u16(payload.from).unwrap())
        .unwrap();
    let end = star_map
        .get(&tools::system_id_to_u16(payload.to).unwrap())
        .unwrap();

    let path = path::calc_path(
        &star_map,
        start,
        end,
        payload.jump_distance,
        payload.optimize.unwrap(),
        payload.use_smart_gates,
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
                use_smart_gates: false,
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
