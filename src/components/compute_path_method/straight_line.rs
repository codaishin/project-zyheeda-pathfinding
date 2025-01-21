use crate::traits::{
	computable_grid::{ComputeGrid, ComputeGridNode},
	compute_path::NewComputer,
};
use std::collections::HashSet;

pub struct StraightLine;

impl NewComputer for StraightLine {
	fn new(_: ComputeGrid, _: HashSet<ComputeGridNode>) -> Self {
		StraightLine
	}
}
