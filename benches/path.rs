use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;
use std::hint::black_box;
use uom::si::f64::*;
use uom::si::length::light_year;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("benches").as_str());
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(1));
    group.sample_size(10);
    group.bench_function(BenchmarkId::new("get_star_map", "data/starmap.bin"), |b| {
        b.iter(|| {
            let _ = evedatacore_route_planner::data::get_star_map(black_box("data/starmap.bin"))
                .unwrap();
        })
    });
    let star_map =
        evedatacore_route_planner::data::get_star_map(black_box("data/starmap.bin")).unwrap();
    for distance_factor in 0..10 {
        let distance: u16 = 50 + distance_factor * 50;
        group.bench_with_input(
            BenchmarkId::new("504ly", &distance),
            &distance,
            |b, distance| {
                let start = star_map
                    .get(&evedatacore_route_planner::tools::system_id_to_u16(30001573).unwrap())
                    .unwrap();
                let end = star_map
                    .get(&evedatacore_route_planner::tools::system_id_to_u16(30013956).unwrap())
                    .unwrap();
                b.iter(|| {
                    let path = evedatacore_route_planner::path::calc_path(
                        &star_map,
                        start,
                        end,
                        black_box(*distance),
                        evedatacore_route_planner::data::PathOptimize::Distance,
                        Some(300),
                    );
                })
            },
        );
    }
    for distance_factor in 0..10 {
        let distance: u16 = 50 + distance_factor * 50;
        group.bench_with_input(
            BenchmarkId::new("4289ly", &distance),
            &distance,
            |b, distance| {
                let start = star_map
                    .get(&evedatacore_route_planner::tools::system_id_to_u16(30017987).unwrap())
                    .unwrap();
                let end = star_map
                    .get(&evedatacore_route_planner::tools::system_id_to_u16(30020622).unwrap())
                    .unwrap();
                b.iter(|| {
                    let path = evedatacore_route_planner::path::calc_path(
                        &star_map,
                        start,
                        end,
                        black_box(*distance),
                        evedatacore_route_planner::data::PathOptimize::Distance,
                        Some(300),
                    );
                })
            },
        );
    }
    for distance_factor in 0..10 {
        let distance: u16 = 50 + distance_factor * 50;
        group.bench_with_input(
            BenchmarkId::new("4618ly", &distance),
            &distance,
            |b, distance| {
                let start = star_map
                    .get(&evedatacore_route_planner::tools::system_id_to_u16(30013115).unwrap())
                    .unwrap();
                let end = star_map
                    .get(&evedatacore_route_planner::tools::system_id_to_u16(30022683).unwrap())
                    .unwrap();
                b.iter(|| {
                    let path = evedatacore_route_planner::path::calc_path(
                        &star_map,
                        start,
                        end,
                        black_box(*distance),
                        evedatacore_route_planner::data::PathOptimize::Distance,
                        Some(300),
                    );
                })
            },
        );
    }
    for distance_factor in 0..10 {
        let distance: u16 = 50 + distance_factor * 50;
        group.bench_with_input(
            BenchmarkId::new("7610ly", &distance),
            &distance,
            |b, distance| {
                let start = star_map
                    .get(&evedatacore_route_planner::tools::system_id_to_u16(30020103).unwrap())
                    .unwrap();
                let end = star_map
                    .get(&evedatacore_route_planner::tools::system_id_to_u16(30022683).unwrap())
                    .unwrap();
                b.iter(|| {
                    let path = evedatacore_route_planner::path::calc_path(
                        &star_map,
                        start,
                        end,
                        black_box(*distance),
                        evedatacore_route_planner::data::PathOptimize::Distance,
                        Some(300),
                    );
                })
            },
        );
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
