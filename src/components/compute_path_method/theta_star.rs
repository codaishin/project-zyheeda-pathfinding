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
	sqrt_2: f32,
	grid: ComputeGrid,
	obstacles: HashSet<ComputeGridNode>,
}

impl ThetaStar {
	const PRECISION: f32 = 10.;
	const NEIGHBORS: &[(i32, i32)] = &[(-1, 0), (0, -1), (0, 1), (1, 0)];

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
		((self.sqrt_2 * short + (long - short)) * Self::PRECISION) as u32
	}

	fn los(&self, a: ComputeGridNode, b: ComputeGridNode) -> bool {
		LineWide::new(a, b).all(|n| !self.obstacles.contains(&n))
	}
}

impl NewComputer for ThetaStar {
	fn new(grid: ComputeGrid, obstacles: HashSet<ComputeGridNode>) -> Self {
		Self {
			grid,
			obstacles,
			sqrt_2: 2_f32.powf(0.5),
		}
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

				let g_neighbor = g_scores.get(&current) + self.distance(current, neighbor);

				if g_neighbor >= g_scores.get(&neighbor) {
					continue;
				}

				let (current, g_neighbor) = match closed.parent(&current) {
					Some(parent) if self.los(*parent, neighbor) => {
						let g_parent = g_scores.get(parent) + self.distance(*parent, neighbor);
						if g_parent <= g_neighbor {
							(*parent, g_parent)
						} else {
							(current, g_neighbor)
						}
					}
					_ => (current, g_neighbor),
				};

				open.push(neighbor, g_neighbor);
				closed.insert(neighbor, current);
				g_scores.insert(neighbor, g_neighbor);
			}
		}

		vec![]
	}
}
