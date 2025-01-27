use crate::traits::{
	computable_grid::{ComputeGrid, ComputeGridNode},
	compute_path::{ComputePath, NewComputer},
};
use std::collections::HashSet;

pub struct ThetaStar;

impl NewComputer for ThetaStar {
	fn new(_: ComputeGrid, _: HashSet<ComputeGridNode>) -> Self {
		Self
	}
}

impl ComputePath for ThetaStar {
	fn draw_connections(&self) -> bool {
		const { true }
	}

	fn path(&self, start: ComputeGridNode, end: ComputeGridNode) -> Vec<ComputeGridNode> {
		vec![start, end]
	}
}
