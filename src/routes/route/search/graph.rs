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
}

#[derive(Debug)]
pub struct Graph {
    pub nodes: NodeMap,
}

impl Graph {
    pub async unsafe fn new(db_pool: PgPool, departure_time: &str) -> Result<Self> {
        let mut nodes: NodeMap = init_nodes(db_pool.clone(), departure_time).await?;
        connect_edges(db_pool, departure_time, &mut nodes).await?;
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

async unsafe fn get_node_edges(db_pool: PgPool, node: &*mut Node) -> Result<Vec<SchedId>> {
    let node_departure_time = NaiveTime::from_str(&(*(*node)).departure_time)?;
    let from_place_sched_id = (*(*node)).from_place_id;
    let sched_destinations: Vec<(SchedId, ToPlaceId)> = query!(
        "SELECT id, to_place_id 
         FROM schedules 
         WHERE 
         from_place_id = $1 AND departure_time >= $2;",
        from_place_sched_id as i32,
        node_departure_time,
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
            node_departure_time
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
    user_dep_time: &str,
    nodes: &mut NodeMap,
) -> Result<()> {
    for (_, &node) in nodes.iter() {
        let edges = get_node_edges(db_pool.clone(), &node).await?;
        (*node).edges =
            calc_edges_weight(db_pool.clone(), user_dep_time, &node, &edges, nodes).await?;
    }

    Ok(())
}

async unsafe fn calc_edges_weight(
    db_pool: PgPool,
    user_dep_time: &str,
    origin_node: &*mut Node,
    edges: &Vec<SchedId>,
    nodes: &NodeMap,
) -> Result<HashMap<NonNull<Node>, Weight>> {
    let mut node_edges = HashMap::new();
    for sched_id in edges {
        let edge = *nodes.get(sched_id).ok_or(anyhow!("Unable to find node"))?;
        let query = query!(
            "SELECT arrival_time 
             FROM schedules 
             WHERE id = $1",
            (sched_id - 1) as i32
        )
        .fetch_all(&db_pool)
        .await?
        .into_iter()
        .map(|record| {
            record
                .arrival_time
                .map_or(None, |rec| Some(rec.format("%H:%M:%S").to_string()))
        })
        .collect::<Vec<Option<String>>>();

        let prev_edge_node_arr_time = {
            query.first().map_or(None, |arr_time| {
                arr_time.as_ref().map(|arr_time| arr_time.to_owned())
            })
        };

        let cost = {
            if let Some(arr_time) = prev_edge_node_arr_time {
                calculate_travel_duration(user_dep_time, &arr_time)?
            } else {
                calculate_travel_duration(
                    user_dep_time,
                    &(*(*origin_node))
                        .weight
                        .expect("sumabog nanaman ulit ang server")
                        .to_string(),
                )?
            }
        };

        node_edges.insert(NonNull::new_unchecked(edge), cost);
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

pub fn parse_time(time: &str) -> Result<usize> {
    Ok(time
        .split(':')
        .take(2)
        .flat_map(|s| s.chars())
        .collect::<String>()
        .parse::<usize>()?)
}

fn calculate_travel_duration(departure_time: &str, arrival_time: &str) -> Result<usize> {
    Ok(parse_time(arrival_time)? - parse_time(departure_time)?)
}
