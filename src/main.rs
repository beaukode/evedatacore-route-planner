use lambda_http::{run, service_fn, tracing, Error};

mod http_handler;
use http_handler::function_handler;

mod shared;
use shared::data;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing for AWS Lambda
    tracing::init_default_subscriber();

    tracing::info!("Loading star map...");

    let path = std::env::var("STARMAP_PATH").unwrap_or_else(|_| String::from("data/starmap.bin"));
    let start = std::time::Instant::now();
    let map: data::StarMap = data::get_star_map(&path)?;
    let map_ref = &map;

    tracing::info!(
        "Star map loaded with {} stars in {}ms",
        map.len(),
        start.elapsed().as_millis()
    );

    let func = service_fn(move |event| async move { function_handler(event, map_ref).await });
    run(func).await
}
