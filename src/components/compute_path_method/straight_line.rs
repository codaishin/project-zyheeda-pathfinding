use crate::traits::{
	computable_grid::{ComputeGrid, ComputeGridNode},
	compute_path::{ComputePath, NewComputer},
};
use std::{collections::HashSet, ops::RangeInclusive};

pub struct StraightLine;

impl NewComputer for StraightLine {
	fn new(_: ComputeGrid, _: HashSet<ComputeGridNode>) -> Self {
		StraightLine
	}
}

impl ComputePath for StraightLine {
	fn draw_connections(&self) -> bool {
		const { true }
	}

	fn path(&self, start: ComputeGridNode, end: ComputeGridNode) -> Vec<ComputeGridNode> {
		Line::new(start, end).collect()
	}
}

/// Uses Bresenham's line algorithm.
/// Sourced from [Wikipedia](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm)
pub struct Line {
	new_node: &'static dyn Fn(i32, i32) -> ComputeGridNode,
	range_high: RangeInclusive<i32>,
	low_stepper: LowStepper,
}

impl Line {
	pub fn new(start: ComputeGridNode, end: ComputeGridNode) -> Self {
		let (low, high, new_node) = Self::layout(start, end);
		let (i_low, d_low) = match low.1 > low.0 {
			true => (1, low.1 - low.0),
			false => (-1, low.0 - low.1),
		};
		let d_high = high.1 - high.0;

		Line {
			new_node,
			range_high: high.0..=high.1,
			low_stepper: LowStepper {
				d: (2 * d_low) - d_high,
				v_low: low.0,
				i_low,
				d_low,
				d_high,
			},
		}
	}

	fn layout(
		start: ComputeGridNode,
		end: ComputeGridNode,
	) -> (Low, High, &'static dyn Fn(i32, i32) -> ComputeGridNode) {
		let dx = (end.x - start.x).abs();
		let dy = (end.y - start.y).abs();
		let is_low = dx > dy;

		match is_low {
			true if start.x < end.x => (Low(start.y, end.y), High(start.x, end.x), &Low::node),
			true => (Low(end.y, start.y), High(end.x, start.x), &Low::node),
			false if start.y < end.y => (Low(start.x, end.x), High(start.y, end.y), &High::node),
			false => (Low(end.x, start.x), High(end.y, start.y), &High::node),
		}
	}
}

impl Iterator for Line {
	type Item = ComputeGridNode;

	fn next(&mut self) -> Option<Self::Item> {
		let v_high = self.range_high.next()?;
		let node = (self.new_node)(self.low_stepper.v_low, v_high);

		self.low_stepper.step();

		Some(node)
	}
}

struct Low(i32, i32);

impl Low {
	fn node(x: i32, y: i32) -> ComputeGridNode {
		ComputeGridNode::new(y, x)
	}
}

struct High(i32, i32);

impl High {
	fn node(x: i32, y: i32) -> ComputeGridNode {
		ComputeGridNode::new(x, y)
	}
}

struct LowStepper {
	v_low: i32,
	i_low: i32,
	d_low: i32,
	d_high: i32,
	d: i32,
}

impl LowStepper {
	fn step(&mut self) {
		if self.d <= 0 {
			self.d += 2 * self.d_low;
			return;
		}

		self.v_low += self.i_low;
		self.d += 2 * (self.d_low - self.d_high);
	}
}
