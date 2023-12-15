use std::sync::Arc;
use async_recursion::async_recursion;
use tokio::sync::Mutex;

struct Graph {
    adjacency: Vec<Vec<usize>>,
    total_nodes: usize
}

#[async_recursion]
async fn get_hamiltonian_cycles(
    node: usize,
    start: usize,
    graph: Arc<Graph>,
    mut path: Vec<usize>,
    results: Arc<Mutex<Vec<Vec<usize>>>>
) {
    //println!("{:?} {}", path, node);
    if path.len() + 1 == graph.total_nodes {
        if graph.adjacency[node].contains(&start) {
            path.push(node);
            path.push(start);
            results.lock().await.push(path);
        }
        return;
    }

    let mut joins = vec![];
    for neighbour in &graph.adjacency[node] {
        if !path.contains(&neighbour) {
            let new_node = neighbour.clone();
            let start_clone = start.clone();
            let graph_clone = graph.clone();
            let results_clone = results.clone();
            let mut new_path = path.clone();
            new_path.push(node);

            joins.push(tokio::task::spawn(async move {
                get_hamiltonian_cycles(
                    new_node,
                    start_clone,
                    graph_clone,
                    new_path,
                    results_clone
                ).await
            }));
        }
    }
    for join in joins {
        join.await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    let graph = Graph {
        adjacency: vec![
            vec![1, 2, 5],         // 0
            vec![0, 2, 5, 4],      // 1
            vec![0, 1, 3],         // 2
            vec![2, 4],            // 3
            vec![1, 3, 5],         // 4
            vec![0, 1, 4],         // 5
        ],
        total_nodes: 6,
    };

    let result = Arc::new(Mutex::new(vec![]));
    get_hamiltonian_cycles(0, 0, Arc::new(graph), vec![], result.clone()).await;

    for path in result.lock().await.to_vec() {
        for el in path {
            print!("{} ", el);
        }
        println!();
    }
}
