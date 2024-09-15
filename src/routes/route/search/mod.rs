use crate::routes::{place::Place, DatabasePool};
use actix_web::{web, HttpResponse, Responder};
use chrono::NaiveTime;
use graph::{Graph, Node};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashSet},
    ptr::NonNull,
};

pub mod graph;

#[derive(Serialize)]
struct ShortestPaths {
    paths: HashSet<Vec<usize>>,
}

pub async fn shortest_paths(
    slug: web::Path<Slug>,
    // query: web::Query<SessionToken>,
    db_pool: web::Data<DatabasePool>,
) -> impl Responder {
    // if let Err(err) = validate_session_token(&query.token, &db_pool.pool).await {
    //     match err {
    //         sqlx::Error::RowNotFound => {
    //             return HttpResponse::Unauthorized().json(RouteRes {
    //                 msg: "unauthorized user".to_string(),
    //             })
    //         }
    //         _ => {
    //             return HttpResponse::InternalServerError().json(RouteRes {
    //                 msg: "server err".to_string(),
    //             })
    //         }
    //     }
    // }

    unsafe {
        let mut graph = Graph::new(db_pool.pool.clone(), &slug.departure_time)
            .await
            .expect("Sumabog ang paggawa ng graph");

        if let Some(paths) = get_shortest_paths(slug.from_place_id, slug.to_place_id, &mut graph) {
            println!("{:?}", paths);
            return HttpResponse::Ok().json(ShortestPaths { paths });
        }
    }

    HttpResponse::Ok().json("haha wala")
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

// get shortest path
unsafe fn get_shortest_paths(
    origin_place_id: i32,
    destination_place_id: i32,
    graph: &mut Graph,
) -> Option<HashSet<Vec<usize>>> {
    if graph.nodes.is_empty() {
        return None;
    }

    // get starting points
    let start_keys = get_node_keys(origin_place_id as usize, graph);
    let start_node_key = start_keys.first()?;

    // perform dijsktra's algorithm
    dijkstra(*start_node_key, graph);

    // get shortest paths by backtracking
    let mut shortest_paths_id: HashSet<Vec<usize>> = HashSet::new();
    let dest_keys = get_node_keys(destination_place_id as usize, graph);
    for start_key in start_keys.iter() {
        for dest_key in dest_keys.iter() {
            // println!("{start_key}: {dest_key}");
            shortest_paths_id.insert(get_shortest_path(*dest_key, *start_key, graph));
        }
    }

    let mut five_shortest_paths = shortest_paths_id
        .iter()
        .enumerate()
        .map(|(ind, vec)| {
            (
                ind,
                vec.iter()
                    .fold(0, |acc, n| acc + (*graph.nodes[n]).travel_duration()),
            )
        })
        .collect::<Vec<(usize, usize)>>();

    five_shortest_paths.sort_by_key(|&(_, travel_duration)| travel_duration);

    let mut res = HashSet::new();
    for (f_ind, _) in five_shortest_paths.iter().enumerate() {
        for (s_ind, vec) in shortest_paths_id.iter().enumerate() {
            if f_ind == s_ind {
                res.insert(vec.clone());
            }
        }
    }

    Some(res)
}

// backtrack
unsafe fn get_shortest_path(dest_key_id: usize, start_node_id: usize, graph: &Graph) -> Vec<usize> {
    let mut path = Vec::new();
    let mut cur_node = graph.nodes.get(&dest_key_id);

    while let Some(node_ref) = cur_node {
        let node = &*node_ref;
        if let Some(prev_node_ptr) = (*(*node)).prev_node {
            let prev_node = &*prev_node_ptr.as_ptr();
            path.push(prev_node.id);

            if prev_node.id == start_node_id {
                break;
            }

            cur_node = graph.nodes.get(&prev_node.id);
        } else {
            break;
        }
    }

    path.reverse();
    path
}

// ACCOUNT FOR START_TIME
unsafe fn dijkstra(start_node_key: usize, graph: &mut Graph) {
    let mut visited_nodes: HashSet<*mut Node> = HashSet::new();
    let mut prio_queue: BinaryHeap<Reverse<*mut Node>> = BinaryHeap::new();

    prio_queue.push(Reverse(graph.nodes[&start_node_key]));

    while let Some(node) = prio_queue.pop() {
        let node = node.0;
        if visited_nodes.contains(&node) {
            continue;
        }

        visited_nodes.insert(node);

        // push node's edges to prio queue
        for (edge, travel_cost) in (*node).edges.iter_mut() {
            let edge = edge.as_ptr();
            if visited_nodes.contains(&edge) {
                continue;
            }

            prio_queue.push(Reverse(edge));

            // update edge's weight
            // check edge's weight
            let edge_weight = (*edge).weight;
            if edge_weight.is_none() || *travel_cost <= edge_weight.unwrap_or(0) {
                (*edge).weight = Some(*travel_cost);
                (*edge).prev_node = Some(NonNull::new_unchecked(node));
            }
        }
    }
}

unsafe fn get_node_keys(origin_place_id: usize, graph: &Graph) -> Vec<usize> {
    let mut ids = graph
        .nodes
        .iter()
        .filter_map(|(&k, &v)| {
            if (*v).from_place_id == origin_place_id {
                Some(k)
            } else {
                None
            }
        })
        .collect::<Vec<usize>>();
    ids.sort();
    ids
}
