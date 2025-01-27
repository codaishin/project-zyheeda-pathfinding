use super::computable_grid::{ComputeGrid, ComputeGridNode};
use std::collections::HashSet;

pub trait NewComputer {
	fn new(grid: ComputeGrid, obstacles: HashSet<ComputeGridNode>) -> Self;
}

pub trait ComputePath {
	fn draw_connections(&self) -> bool;
	fn path(&self, start: ComputeGridNode, end: ComputeGridNode) -> Vec<ComputeGridNode>;
}
