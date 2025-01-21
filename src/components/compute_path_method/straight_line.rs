use crate::traits::{
	computable_grid::{ComputeGrid, ComputeGridNode},
	compute_path::{ComputePath, NewComputer},
};
use std::collections::HashSet;

pub struct StraightLine;

impl NewComputer for StraightLine {
	fn new(_: ComputeGrid, _: HashSet<ComputeGridNode>) -> Self {
		StraightLine
	}
}

impl ComputePath for StraightLine {
	fn path(&self, start: ComputeGridNode, end: ComputeGridNode) -> Vec<ComputeGridNode> {
		vec![start, end]
	}
}
