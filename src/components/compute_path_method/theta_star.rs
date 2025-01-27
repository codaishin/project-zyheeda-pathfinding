use super::{
	a_star::{ClosedList, GScores, OpenList},
	straight_line::Line,
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
	const NEIGHBORS: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];

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

	fn distance(a: &ComputeGridNode, b: &ComputeGridNode) -> u32 {
		a.x.abs_diff(b.x) + a.y.abs_diff(b.y)
	}

	fn los(&self, a: ComputeGridNode, b: ComputeGridNode) -> bool {
		Line::new(a, b).all(|n| !self.obstacles.contains(&n))
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
		let mut open = OpenList::new(start, end);
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

				let g_neighbor = g_scores.get(&current) + Self::distance(&current, &neighbor);

				if g_neighbor >= g_scores.get(&neighbor) {
					continue;
				}

				let (current, g_neighbor) = match closed.parent(&current) {
					Some(parent) if self.los(*parent, neighbor) => {
						let g_parent = g_scores.get(parent) + Self::distance(parent, &neighbor);
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
