use anyhow::{anyhow, Result};
use chrono::NaiveTime;
use sqlx::{query, PgPool};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ptr::NonNull;
use std::str::FromStr;

type SchedId = usize;
type ToPlaceId = usize;
type Weight = usize;
type NodeMap = HashMap<SchedId, *mut Node>;

#[derive(Debug)]
pub struct Node {
    pub id: usize,
    pub from_place_id: usize,
    pub to_place_id: usize,
    pub departure_time: String,
    pub arrival_time: String,
    pub prev_node: Option<NonNull<Node>>,
    pub edges: HashMap<NonNull<Node>, Weight>,
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

    pub fn travel_duration(&self) -> Result<usize> {
        calculate_travel_duration(&self.departure_time, &self.arrival_time)
    }
}

#[derive(Debug)]
pub struct Graph {
    pub nodes: NodeMap,
}

impl Graph {
    pub async unsafe fn new(db_pool: PgPool, departure_time: &str) -> Result<Self> {
        let mut nodes: NodeMap = init_nodes(db_pool.clone(), departure_time).await?;
        connect_edges(db_pool, &mut nodes, departure_time).await?;
        Ok(Self { nodes })
    }
}

impl Drop for Graph {
    fn drop(&mut self) {
        for (_, node) in self.nodes.iter() {
            unsafe {
                let _ = Box::from_raw(*node);
            }
        }
    }
}

async unsafe fn get_node_edges(
    db_pool: PgPool,
    departure_time: &str,
    node: &*mut Node,
) -> Result<Vec<SchedId>> {
    let departure_time = NaiveTime::from_str(departure_time)?;
    let from_place_sched_id = (*(*node)).from_place_id;
    let sched_destinations: Vec<(SchedId, ToPlaceId)> = query!(
        "SELECT id, to_place_id 
         FROM schedules 
         WHERE 
         from_place_id = $1 AND departure_time >= $2;",
        from_place_sched_id as i32,
        departure_time
    )
    .fetch_all(&db_pool.clone())
    .await?
    .into_iter()
    .map(|n| (n.id as usize, n.to_place_id as usize))
    .collect();

    let mut edges: Vec<SchedId> = Vec::new();
    for (sched_id, to_place_id) in sched_destinations.into_iter() {
        let query = query!(
            "SELECT id 
             FROM schedules 
             WHERE id = $1 
             AND from_place_id = $2 
             AND departure_time >= $3;",
            (sched_id + 1) as i32,
            (to_place_id) as i32,
            departure_time
        )
        .fetch_all(&db_pool)
        .await?
        .into_iter()
        .map(|record| record.id as usize)
        .collect::<Vec<usize>>();

        edges.extend(query.into_iter());
    }

    Ok(edges)
}

async unsafe fn connect_edges(
    db_pool: PgPool,
    nodes: &mut NodeMap,
    departure_time: &str,
) -> Result<()> {
    for (_, &node) in nodes.iter() {
        let edges = get_node_edges(db_pool.clone(), departure_time, &node).await?;
        let edges = calc_edges_weight(&edges, nodes).await?;
        (*node).edges = edges;
    }

    Ok(())
}

async unsafe fn calc_edges_weight(
    edges: &Vec<SchedId>,
    nodes: &NodeMap,
) -> Result<HashMap<NonNull<Node>, Weight>> {
    let mut node_edges = HashMap::new();
    for sched_id in edges {
        let node = *nodes.get(sched_id).ok_or(anyhow!("Unable to find node"))?;
        let departure_time = &(*node).departure_time;
        let arrival_time = &(*node).arrival_time;
        let travel_duration = calculate_travel_duration(departure_time, arrival_time)?;
        let node = NonNull::new_unchecked(node);

        node_edges.insert(node, travel_duration);
    }

    Ok(node_edges)
}

async unsafe fn init_nodes(db_pool: PgPool, departure_time: &str) -> Result<NodeMap> {
    let mut nodes = HashMap::new();
    let queries = query!(
        "SELECT id, from_place_id, to_place_id, departure_time, arrival_time 
         FROM schedules 
         WHERE departure_time >= $1;",
        NaiveTime::from_str(departure_time)?
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
                .ok_or(anyhow!("time explodes"))?
                .format("%H:%M:%S")
                .to_string(),
            query
                .arrival_time
                .ok_or(anyhow!("time explodes"))?
                .format("%H:%M:%S")
                .to_string(),
        )));
        nodes.insert((*new_node).id, new_node);
    }

    Ok(nodes)
}

pub fn calculate_travel_duration(departure_time: &str, arrival_time: &str) -> Result<usize> {
    let parse_time = |time: &str| -> Result<usize> {
        Ok(time
            .split(':')
            .take(2)
            .flat_map(|s| s.chars())
            .collect::<String>()
            .parse::<usize>()?)
    };

    Ok(parse_time(arrival_time)? - parse_time(departure_time)?)
}
