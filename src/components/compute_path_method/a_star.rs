use crate::traits::{
	computable_grid::{ComputeGrid, ComputeGridNode},
	compute_path::{ComputePath, NewComputer},
};
use std::collections::HashSet;

pub struct AStar;

impl NewComputer for AStar {
	fn new(_: ComputeGrid, _: HashSet<ComputeGridNode>) -> Self {
		AStar
	}
}

impl ComputePath for AStar {
	fn path(&self, start: ComputeGridNode, end: ComputeGridNode) -> Vec<ComputeGridNode> {
		vec![start, end]
	}
}
