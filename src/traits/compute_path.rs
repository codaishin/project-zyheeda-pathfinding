use super::computable_grid::{ComputeGrid, ComputeGridNode};
use std::collections::HashSet;

pub trait NewComputer {
	fn new(grid: ComputeGrid, obstacles: HashSet<ComputeGridNode>) -> Self;
}
