use chrono::NaiveTime;
use sqlx::{query, PgPool};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ptr::NonNull;
use std::str::FromStr;

#[derive(Debug)]
pub struct Node {
    pub id: usize,
    pub from_place_id: usize,
    pub to_place_id: usize,
    pub departure_time: String,
    pub arrival_time: String,
    pub prev_node: Option<NonNull<Node>>,
    pub edges: HashMap<NonNull<Node>, usize>,
    pub weight: Option<usize>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.weight.partial_cmp(&other.weight)
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.weight.cmp(&other.weight)
    }
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

    pub fn travel_duration(&self) -> usize {
        calculate_travel_duration(&self.departure_time, &self.arrival_time)
    }
}

// Create graph and nodes
#[derive(Debug)]
pub struct Graph {
    pub nodes: HashMap<usize, *mut Node>,
}

// REFACTOR
impl Graph {
    pub async unsafe fn new(db_pool: PgPool, departure_time: &str) -> Result<Self, sqlx::Error> {
        // For each query, create new nodes and add it to nodes vector
        let mut nodes: HashMap<usize, *mut Node> = HashMap::new();
        let queries = query!(
            "SELECT id, from_place_id, to_place_id, departure_time, arrival_time 
             FROM schedules 
             WHERE departure_time >= $1;",
            NaiveTime::from_str(departure_time).expect(&format!(
                "Sumabog ang conversion ng oras {}",
                departure_time
            ))
        )
        .fetch_all(&db_pool)
        .await?;
        for query in queries {
            // DO CLEANUP
            let new_node = Box::into_raw(Box::new(Node::new(
                query.id as usize,
                query.from_place_id as usize,
                query.to_place_id as usize,
                query
                    .departure_time
                    .expect("Time explodes")
                    .format("%H:%M:%S")
                    .to_string(),
                query
                    .arrival_time
                    .expect("Time explodes")
                    .format("%H:%M:%S")
                    .to_string(),
            )));
            nodes.insert((*new_node).id, new_node);
        }

        // Fill edges of nodes
        // Get edges of node id
        let mut node_edges_id: HashMap<usize, Vec<i32>> = HashMap::new();
        for (id, node) in nodes.iter() {
            // query appropriate time
            // fill node according to appropriate schedule id
            let departure_time = &(*(*node)).departure_time;
            let edge_candidate_stations_id = query!(
                "SELECT id, to_place_id 
                 FROM schedules 
                 WHERE from_place_id = $1 
                 AND departure_time >= $2;",
                (*(*node)).from_place_id as i32,
                NaiveTime::from_str(departure_time).expect(&format!(
                    "Sumabog ang conversion ng oras {}",
                    (*(*node)).departure_time
                ))
            )
            .fetch_all(&db_pool)
            .await?
            .iter()
            .map(|rec| (rec.id, rec.to_place_id))
            .collect::<Vec<(i32, i32)>>();

            let mut kandydate = Vec::new();
            for (sched_origin_id, edge_to_place_id) in edge_candidate_stations_id {
                let mut query: Vec<i32> = query!(
                    "SELECT id from schedules 
                     WHERE id = $1 AND from_place_id = $2 AND departure_time >= $3;",
                    sched_origin_id + 1,
                    edge_to_place_id,
                    NaiveTime::from_str(departure_time).expect(&format!(
                        "Sumabog ang conversion ng oras {}",
                        (*(*node)).departure_time
                    ))
                )
                .fetch_all(&db_pool)
                .await?
                .iter()
                .map(|rec| rec.id)
                .collect();

                kandydate.append(&mut query);
            }

            node_edges_id.insert(*id, kandydate);
        }

        // fill edges of the current id by node's address
        for (node_id, edge_ids) in node_edges_id {
            let node = *(nodes.get(&node_id).expect("Boom"));
            let departure_time = &(*node).departure_time;
            let arrival_time = &(*node).arrival_time;
            for edge_id in edge_ids {
                let edge = nodes.get(&(edge_id as usize)).expect("Kaboom");
                let travel_duration = calculate_travel_duration(departure_time, arrival_time);
                (*node)
                    .edges
                    .insert(NonNull::new_unchecked(*edge), travel_duration);
            }
        }

        Ok(Self { nodes })
    }
}

pub fn calculate_travel_duration(departure_time: &str, arrival_time: &str) -> usize {
    let parse_time = |time: &str| -> usize {
        time.split(':')
            .take(2)
            .flat_map(|s| s.chars())
            .collect::<String>()
            .parse::<usize>()
            .expect("Conversion of time explodes part 2")
    };

    parse_time(arrival_time) - parse_time(departure_time)
}
