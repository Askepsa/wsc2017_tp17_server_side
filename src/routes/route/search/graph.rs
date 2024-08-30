use actix_web::web;
use chrono::NaiveTime;
use sqlx::{query, query_as, PgPool};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::ptr::NonNull;
use std::str::FromStr;

pub struct Node {
    id: usize,
    from_place_id: usize,
    to_place_id: usize,
    departure_time: String,
    arrival_time: String,
    prev_node: Option<NonNull<Node>>,
    edges: HashMap<NonNull<Node>, usize>,
    weight: Option<usize>,
}

impl Node {
    fn new(
        id: usize,
        from_place_id: usize,
        to_place_id: usize,
        departure_time: String,
        arrival_time: String,
    ) -> Self {
        Self {
            id,
            from_place_id,
            to_place_id,
            departure_time,
            arrival_time,
            prev_node: None,
            edges: HashMap::new(),
            weight: None,
        }
    }
}

// Create graph and nodes
pub struct Graph {
    nodes: Vec<Node>,
}

impl Graph {
    pub async fn new(db_pool: web::Data<PgPool>) -> Result<Self, sqlx::Error> {
        // For each query, create new nodes and add it to nodes vector
        let mut nodes = HashMap::new(); // make value a pointer of node
        let queries = query!(
            "SELECT id, from_place_id, to_place_id, departure_time, arrival_time FROM schedules;",
        )
        .fetch_all(db_pool.as_ref())
        .await?;
        for query in queries {
            let new_node = Node::new(
                query.id as usize,
                query.from_place_id as usize,
                query.to_place_id as usize,
                query.departure_time.expect("Time explodes").to_string(),
                query.arrival_time.expect("Time explodes").to_string(),
            );
            nodes.insert(new_node.id, new_node);
        }

        // Fill edges of nodes
        // Get edges of node id

        let mut node_edges_id = HashMap::new();
        for (k, node) in nodes.iter() {
            // query appropriate time
            // fill node according to appropriate schedule id
            let candidate_edges = query!(
                "SELECT id, from_place_id, to_place_id 
                 FROM schedules
                 WHERE (from_place_id = $1 OR to_place_id = $2)
                 AND departure_time >= $3;",
                node.from_place_id as i32,
                node.to_place_id as i32,
                NaiveTime::from_str(&node.departure_time).expect("Sumabog ang conversion ng oras")
            )
            .fetch_all(db_pool.as_ref())
            .await?;

            // find edges of the current id
            let mut edges = Vec::new();
            for (indx, edge) in candidate_edges.iter().enumerate() {
                if let Some(next_edge) = candidate_edges.get(indx + 1) {
                    if edge.to_place_id == next_edge.from_place_id {
                        edges.push(next_edge.id);
                    }
                }
            }
            node_edges_id.insert(node.id, edges);
        }

        // fill edges of the current id by node's address
        for (node_id, edges) in node_edges_id {

        }

        todo!()
    }
}
