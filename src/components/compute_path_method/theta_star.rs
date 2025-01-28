use super::{
	a_star::{ClosedList, GScores, OpenList},
	straight_line_wide::LineWide,
};
use crate::traits::{
	computable_grid::{ComputeGrid, ComputeGridNode},
	compute_path::{ComputePath, NewComputer},
};
use std::collections::HashSet;

pub struct ThetaStar {
	grid: ComputeGrid,
	obstacles: HashSet<ComputeGridNode>,
}

impl ThetaStar {
	const SQRT_2: f32 = 1.4;
	const PRECISION: f32 = 10.;
	const NEIGHBORS: &[(i32, i32)] = &[
		(-1, -1),
		(-1, 0),
		(-1, 1),
		(0, -1),
		(0, 1),
		(1, -1),
		(1, 0),
		(1, 1),
	];

	fn neighbors<'a>(
		&'a self,
		center: &'a ComputeGridNode,
	) -> impl Iterator<Item = ComputeGridNode> + 'a {
		Self::NEIGHBORS
			.iter()
			.map(|(x, y)| ComputeGridNode::new(center.x + x, center.y + y))
			.filter(|ComputeGridNode { x, y }| {
				x <= &self.grid.max.x
					&& x >= &self.grid.min.x
					&& y <= &self.grid.max.y
					&& y >= &self.grid.min.y
			})
	}

	fn distance(&self, a: ComputeGridNode, b: ComputeGridNode) -> u32 {
		let d_x = a.x.abs_diff(b.x) as f32;
		let d_y = a.y.abs_diff(b.y) as f32;
		let (long, short) = match d_x > d_y {
			true => (d_x, d_y),
			false => (d_y, d_x),
		};
		((Self::SQRT_2 * short + (long - short)) * Self::PRECISION) as u32
	}

	fn los(&self, a: ComputeGridNode, b: ComputeGridNode) -> bool {
		LineWide::new(a, b).all(|n| !self.obstacles.contains(&n))
	}

	fn vertex(
		&self,
		closed: &ClosedList,
		g_scores: &GScores,
		current: ComputeGridNode,
		neighbor: ComputeGridNode,
	) -> Option<(ComputeGridNode, u32)> {
		match closed.parent(&current) {
			Some(parent) if self.los(*parent, neighbor) => self.relax(g_scores, *parent, neighbor),
			_ if self.los(current, neighbor) => self.relax(g_scores, current, neighbor),
			_ => None,
		}
	}

	fn relax(
		&self,
		g_scores: &GScores,
		current: ComputeGridNode,
		neighbor: ComputeGridNode,
	) -> Option<(ComputeGridNode, u32)> {
		let g = g_scores.get(&current) + self.distance(current, neighbor);

		if g >= g_scores.get(&neighbor) {
			return None;
		}

		Some((current, g))
	}
}

impl NewComputer for ThetaStar {
	fn new(grid: ComputeGrid, obstacles: HashSet<ComputeGridNode>) -> Self {
		Self { grid, obstacles }
	}
}

impl ComputePath for ThetaStar {
	fn draw_connections(&self) -> bool {
		const { true }
	}

	fn path(&self, start: ComputeGridNode, end: ComputeGridNode) -> Vec<ComputeGridNode> {
		let dist_f = |a, b| self.distance(a, b);
		let mut open = OpenList::new(start, end, &dist_f);
		let mut closed = ClosedList::new(start);
		let mut g_scores = GScores::new(start);

		while let Some(current) = open.pop_lowest_f() {
			if current == end {
				return closed.walk_back_from(&current).collect();
			}

			for neighbor in self.neighbors(&current) {
				if self.obstacles.contains(&neighbor) {
					continue;
				}

				let Some((current, g)) = self.vertex(&closed, &g_scores, current, neighbor) else {
					continue;
				};

				open.push(neighbor, g);
				closed.insert(neighbor, current);
				g_scores.insert(neighbor, g);
			}
		}

		vec![]
	}
}
