// https://docs.rs/crate/pathfinding/latest/source/src/directed/astar.rs
// modified to return both nodes and edges
use indexmap::map::Entry::{Occupied, Vacant};
use num_traits::Zero;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::hash::Hash;
use std::time::{Duration, Instant};

use indexmap::IndexMap;
use log::info;
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

pub struct Stats {
    pub total_time: Duration,
    pub heuristic_spend: Duration,
    pub successors_spend: Duration,
    pub loop_spend: Duration,
    pub visited: u64,
}

pub enum PathFindResult<N, C> {
    Found((Vec<N>, C, Stats)),
    NotFound(Stats),
    Timeout(Stats),
}

pub fn astar<N, C, FN, IN, FH, FS>(
    start: &N,
    mut successors: FN,
    mut heuristic: FH,
    mut success: FS,
    timeout: Option<u64>,
) -> PathFindResult<N, C>
where
    N: Eq + Hash + Clone,
    C: Zero + Ord + Copy,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FH: FnMut(&N) -> C,
    FS: FnMut(&N) -> bool,
{
    let mut stats = Stats {
        total_time: Duration::from_secs(0),
        heuristic_spend: Duration::from_secs(0),
        successors_spend: Duration::from_secs(0),
        loop_spend: Duration::from_secs(0),
        visited: 0,
    };
    let start_time = Instant::now();
    let mut to_see = BinaryHeap::new();
    to_see.push(SmallestCostHolder {
        estimated_cost: Zero::zero(),
        cost: Zero::zero(),
        index: 0,
    });
    let mut parents: FxIndexMap<N, (usize, C)> = FxIndexMap::default();
    parents.insert(start.clone(), (usize::MAX, Zero::zero()));
    while let Some(SmallestCostHolder { cost, index, .. }) = to_see.pop() {
        if timeout.is_some() && start_time.elapsed().as_secs() >= timeout.unwrap() {
            stats.total_time = start_time.elapsed();
            return PathFindResult::Timeout(stats);
        }
        stats.visited += 1;
        let successors = {
            let (node, &(_, c)) = parents.get_index(index).unwrap(); // Cannot fail
            if success(node) {
                let path = reverse_path(&parents, |&(p, _)| p, index);
                stats.total_time = start_time.elapsed();
                return PathFindResult::Found((path, cost, stats));
            }
            // We may have inserted a node several time into the binary heap if we found
            // a better way to access it. Ensure that we are currently dealing with the
            // best path and discard the others.
            if cost > c {
                continue;
            }
            let start_time = Instant::now();
            let r = successors(node);
            stats.successors_spend += start_time.elapsed();
            r
        };

        let start_time = Instant::now();
        let mut new_nodes = Vec::new();
        for (successor, move_cost) in successors {
            let new_cost = cost + move_cost;
            let h; // heuristic(&successor)
            let n; // index for successor
            match parents.entry(successor) {
                Vacant(e) => {
                    let start_time = Instant::now();
                    h = heuristic(e.key());
                    stats.heuristic_spend += start_time.elapsed();
                    n = e.index();
                    e.insert((index, new_cost));
                }
                Occupied(mut e) => {
                    if e.get().1 > new_cost {
                        let start_time = Instant::now();
                        h = heuristic(e.key());
                        stats.heuristic_spend += start_time.elapsed();
                        n = e.index();
                        e.insert((index, new_cost));
                    } else {
                        continue;
                    }
                }
            }
            new_nodes.push(SmallestCostHolder {
                estimated_cost: new_cost + h,
                cost: new_cost,
                index: n,
            });
        }
        stats.loop_spend += start_time.elapsed();
        to_see.extend(new_nodes);
    }
    stats.total_time = start_time.elapsed();
    PathFindResult::NotFound(stats)
}

#[allow(clippy::needless_collect)]
fn reverse_path<N, V, F>(parents: &FxIndexMap<N, V>, mut parent: F, start: usize) -> Vec<N>
where
    N: Eq + Hash + Clone,
    F: FnMut(&V) -> usize,
{
    let mut i = start;
    let path = std::iter::from_fn(|| {
        parents.get_index(i).map(|(node, value)| {
            i = parent(value);
            node
        })
    })
    .collect::<Vec<&N>>();
    // Collecting the going through the vector is needed to revert the path because the
    // unfold iterator is not double-ended due to its iterative nature.
    path.into_iter().rev().cloned().collect()
}

struct SmallestCostHolder<K> {
    estimated_cost: K,
    cost: K,
    index: usize,
}

impl<K: PartialEq> PartialEq for SmallestCostHolder<K> {
    fn eq(&self, other: &Self) -> bool {
        self.estimated_cost.eq(&other.estimated_cost) && self.cost.eq(&other.cost)
    }
}

impl<K: PartialEq> Eq for SmallestCostHolder<K> {}

impl<K: Ord> PartialOrd for SmallestCostHolder<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord> Ord for SmallestCostHolder<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.estimated_cost.cmp(&self.estimated_cost) {
            Ordering::Equal => self.cost.cmp(&other.cost),
            s => s,
        }
    }
}
