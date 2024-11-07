use crate::routes::{place::Place, DatabasePool};
use actix_web::{web, HttpResponse, Responder};
use anyhow::{anyhow, Result};
use chrono::NaiveTime;
use graph::{parse_time, Graph, Node};
use serde::{Deserialize, Serialize};
use std::{cmp::Reverse, collections::BinaryHeap, ptr::NonNull};

pub mod graph;

pub async fn shortest_paths(
    slug: web::Path<Slug>,
    db_pool: web::Data<DatabasePool>,
) -> impl Responder {
    let mut graph = unsafe {
        match Graph::new(db_pool.pool.clone(), &slug.departure_time).await {
            Ok(graph) => graph,
            Err(_) => return HttpResponse::BadRequest().json("invalid request"),
        }
    };

    let paths = unsafe {
        calculate_shortest_paths(
            slug.from_place_id,
            slug.to_place_id,
            &slug.departure_time,
            &mut graph,
        )
    };

    match paths {
        Ok(paths) => match paths {
            Some(paths) => return HttpResponse::Ok().json(ShortestPaths { paths }),
            _ => HttpResponse::NotFound().json("haha wala"),
        },
        _ => HttpResponse::InternalServerError().json("sumabog ang server"),
    }
}

#[derive(Deserialize)]
pub struct Slug {
    from_place_id: i32,
    to_place_id: i32,
    departure_time: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseSchedule {
    id: i32,
    line: i32,
    from_place_id: i32,
    to_place_id: i32,
    departure_time: NaiveTime,
    arrival_time: NaiveTime,
    travel_time: NaiveTime,
    from_place: Place,
    to_place: Place,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Res {
    schedules: Vec<ResponseSchedule>,
}

#[derive(Serialize)]
struct ShortestPaths {
    paths: Vec<(Vec<usize>, usize)>,
}

// aysuin nalang to ig
unsafe fn calculate_shortest_paths(
    origin_place_id: i32,
    destination_place_id: i32,
    departure_time: &str,
    graph: &mut Graph,
) -> Result<Option<Vec<(Vec<usize>, usize)>>> {
    let starting_points =
        find_from_place_sched_ids(origin_place_id as usize, departure_time, graph);
    let dest_points =
        find_from_place_sched_ids(destination_place_id as usize, departure_time, graph);
    dijkstra(departure_time, graph)?;
    Ok(collect_shortest_paths(starting_points, dest_points, graph))
}

unsafe fn collect_shortest_paths(
    starting_points: Vec<usize>,
    dest_points: Vec<usize>,
    graph: &Graph,
) -> Option<Vec<(Vec<usize>, usize)>> {
    let mut sorted_dest_place_ids: Vec<(usize, usize)> = dest_points
        .into_iter()
        .filter_map(|dest_id| {
            let weight = (*graph.nodes[&dest_id]).weight;
            weight.map(|w| (dest_id, w))
        })
        .collect();
    sorted_dest_place_ids.sort_by_key(|(_, weight)| *weight);

    let shortest_paths: Vec<(Vec<usize>, usize)> =
        sorted_dest_place_ids
            .into_iter()
            .fold(Vec::new(), |mut acc, (id, weight)| {
                let mut shortest_path = Vec::new();
                let goal_node = graph.nodes[&id];
                let mut prev_node = (*goal_node).prev_node;
                shortest_path.push(id);
                while let Some(node) = prev_node {
                    let sched_id = (*node.as_ptr()).id;
                    shortest_path.push(sched_id);
                    prev_node = (*node.as_ptr()).prev_node;
                    if starting_points.contains(&sched_id) {
                        break;
                    }
                }
                shortest_path.reverse();
                if !shortest_path.is_empty() {
                    acc.push((shortest_path, weight));
                }

                acc
            });

    if shortest_paths.is_empty() {
        return None;
    }

    Some(shortest_paths)
}

unsafe fn find_from_place_sched_ids(
    origin_place_id: usize,
    departure_time: &str,
    graph: &mut Graph,
) -> Vec<usize> {
    let starting_points = graph
        .nodes
        .iter()
        .map(|(&sched_id, &node)| -> Result<usize> {
            let node_from_place_id = (*node).from_place_id;
            let node_dep_time = parse_time(&(*node).departure_time)?;
            let departure_time = parse_time(departure_time)?;
            if node_dep_time >= departure_time && origin_place_id == node_from_place_id {
                Ok(sched_id)
            } else {
                Err(anyhow!("wala kang nakita"))
            }
        });

    starting_points.filter_map(|res| res.ok()).collect()
}

unsafe fn dijkstra(departure_time: &str, graph: &mut Graph) -> Result<()> {
    let nodes = &mut graph.nodes;
    let min_node_sched_id = match nodes.keys().copied().min() {
        Some(sched_id) => sched_id,
        None => return Ok(()),
    };

    let first_to_visit_node = nodes[&min_node_sched_id];
    let first_to_visit_node_dep_time = &(*first_to_visit_node).departure_time;
    (*first_to_visit_node).weight =
        Some(parse_time(first_to_visit_node_dep_time)? - parse_time(departure_time)?);

    let mut binary_heap: BinaryHeap<Reverse<*mut Node>> = BinaryHeap::new();
    binary_heap.push(Reverse(first_to_visit_node));

    while let Some(node) = binary_heap.pop() {
        let prev_node = node.0;
        let prev_node_weight = match (*prev_node).weight {
            Some(w) => w,
            None => continue,
        };

        let edges = &(*prev_node).edges;
        for (&edge, &cost) in edges {
            let travel_weight = prev_node_weight + cost;
            let edge = edge.as_ptr();
            let edge_weight = (*edge).weight;
            if edge_weight.map_or(true, |w| w > travel_weight) {
                (*edge).weight = Some(travel_weight);
                (*edge).prev_node = Some(NonNull::new_unchecked(prev_node));
                if edge_weight.is_none() {
                    binary_heap.push(Reverse(edge));
                }
            }
        }
    }

    Ok(())
}
