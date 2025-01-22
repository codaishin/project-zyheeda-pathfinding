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
	/// Uses Bresenham's line algorithm.
	/// Copied from [Wikipedia](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm)
	fn path(&self, start: ComputeGridNode, end: ComputeGridNode) -> Vec<ComputeGridNode> {
		let x0 = start.x;
		let y0 = start.y;
		let x1 = end.x;
		let y1 = end.y;

		let dx = (x1 - x0).abs();
		let dy = (y1 - y0).abs();

		match dx > dy {
			true if x0 > x1 => plot_line_low(x1, y1, x0, y0),
			true => plot_line_low(x0, y0, x1, y1),
			false if y0 > y1 => plot_line_high(x1, y1, x0, y0),
			false => plot_line_high(x0, y0, x1, y1),
		}
	}
}

fn plot_line_low(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<ComputeGridNode> {
	let mut points = vec![];

	let dx = x1 - x0;
	let mut dy = y1 - y0;
	let mut yi = 1;
	if dy < 0 {
		yi = -1;
		dy = -dy;
	}
	let mut d = (2 * dy) - dx;
	let mut y = y0;

	for x in x0..=x1 {
		points.push(ComputeGridNode::new(x, y));
		if d > 0 {
			y += yi;
			d += 2 * (dy - dx);
		} else {
			d += 2 * dy;
		}
	}

	points
}

fn plot_line_high(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<ComputeGridNode> {
	let mut points = vec![];

	let mut dx = x1 - x0;
	let dy = y1 - y0;
	let mut xi = 1;
	if dx < 0 {
		xi = -1;
		dx = -dx;
	}
	let mut d = (2 * dx) - dy;
	let mut x = x0;

	for y in y0..=y1 {
		points.push(ComputeGridNode::new(x, y));
		if d > 0 {
			x += xi;
			d += 2 * (dx - dy);
		} else {
			d += 2 * dx;
		}
	}

	points
}
