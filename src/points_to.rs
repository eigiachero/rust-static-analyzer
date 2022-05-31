use std::collections::HashMap;

use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::Dfs;

pub struct PointsToGraph {
    pub graph: Graph::<u32,()>,
    variables: HashMap<u32, NodeIndex>
}

impl PointsToGraph {
    pub fn new() -> PointsToGraph {
        PointsToGraph {
            graph: Graph::new(),
            variables: HashMap::new(),
        }
    }

    pub fn get_variable(&self, a: u32) -> NodeIndex<u32> {
        self.variables.get(&a).unwrap().to_owned()
    }

    pub fn constant(&mut self, a: u32) {
        self.variables.insert(a, self.graph.add_node(a));
        // println!("{:?} | Added {}", self.variables, a);
    }

    pub fn points_to(&mut self, a: u32, b: u32) {
        let node = self.graph.add_node(a);
        self.variables.insert(a, node);
        self.graph.add_edge(node, self.get_variable(b), ());
        // println!("{:?} | {} points to {}", self.variables, a, b);
    }

    pub fn are_alias(&self, a:u32, b:u32) -> bool {
        let mut nodes_from_a = Vec::new();
        let (a, b) = (self.get_variable(a), self.get_variable(b));

        // Get all extended neighboor nodes starting from A
        let mut dfs_a = Dfs::new(&self.graph, a);
        while let Some(node) = dfs_a.next(&self.graph) {
            nodes_from_a.push(node);
        }

        // Check if A and B has a common extended neighboor node and return if that's the case
        let mut dfs_b = Dfs::new(&self.graph, b);
        while let Some(node) = dfs_b.next(&self.graph) {
            if nodes_from_a.contains(&node) {
                return true
            }
        }

        // Else return false
        false
    }

}