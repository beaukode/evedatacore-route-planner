use lambda_http::{tracing, Body, Error, Request, RequestExt, Response};
use uom::si::f64::*;
use uom::si::length::light_year;

use crate::shared::{astar, data, path, raw, tools};

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
pub(crate) async fn function_handler(
    event: Request,
    star_map: &data::StarMap,
) -> Result<Response<Body>, Error> {
    // Extract some useful information from the request
    let params = event.query_string_parameters_ref();
    let (from, to, distance) = match params {
        Some(params) => {
            match (
                params.first("from"),
                params.first("to"),
                params.first("distance"),
            ) {
                (Some(from), Some(to), Some(distance)) => (Some(from), Some(to), Some(distance)),
                _ => (None, None, None), // Or handle missing parameters differently
            }
        }
        None => (None, None, None),
    };

    // Then check if both are present
    match (from, to, distance) {
        (Some(from_val), Some(to_val), Some(distance_val)) => {
            let message = format!(
                "Calculate route from {from_val} to {to_val} with max distance {distance_val}"
            );

            let from_star = star_map
                .get(&tools::system_id_to_u16(from_val.parse::<u32>().unwrap()).unwrap())
                .unwrap();
            let end = star_map
                .get(&tools::system_id_to_u16(to_val.parse::<u32>().unwrap()).unwrap())
                .unwrap();
            let distance = distance_val.parse::<u16>().unwrap();
            // let jump_distance: Length = Length::new::<light_year>(200.0);
            let path = path::calc_path(
                &star_map,
                from_star,
                end,
                distance,
                path::PathOptimize::Distance,
                false,
                Some(300),
            );

            let mut path_str = String::new();
            match path {
                astar::PathFindResult::Found((connections, cost, stats)) => {
                    for conn in connections {
                        path_str.push_str(&format!("{} -> ", tools::u16_to_system_id(conn.target)));
                    }
                    path_str.push_str(&format!(
                        "[{} {} {}]",
                        cost,
                        stats.total_time.as_millis(),
                        stats.visited
                    ));
                }
                astar::PathFindResult::NotFound(stats) => {
                    path_str.push_str("No path found");
                }
                astar::PathFindResult::Timeout(stats) => {
                    path_str.push_str("Path finding timed out");
                }
            }

            // Return something that implements IntoResponse.
            // It will be serialized to the right response event automatically by the runtime
            let resp = Response::builder()
                .status(200)
                .header("content-type", "text/html")
                .body(path_str.into())
                .map_err(Box::new)?;
            Ok(resp)
        }
        _ => {
            // Handle missing parameters
            return Ok(Response::builder()
                .status(400)
                .body("Invalid parameters".into())?);
        }
    }
}
