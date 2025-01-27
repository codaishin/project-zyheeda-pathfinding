use crate::traits::{
	computable_grid::{ComputeGrid, ComputeGridNode},
	compute_path::{ComputePath, NewComputer},
};
use std::{
	cmp::{Ordering, Reverse},
	collections::{BinaryHeap, HashMap, HashSet},
};

pub struct AStar {
	grid: ComputeGrid,
	obstacles: HashSet<ComputeGridNode>,
}

impl AStar {
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
}

impl NewComputer for AStar {
	fn new(grid: ComputeGrid, obstacles: HashSet<ComputeGridNode>) -> Self {
		Self { grid, obstacles }
	}
}

impl ComputePath for AStar {
	fn draw_connections(&self) -> bool {
		const { true }
	}

	fn path(&self, start: ComputeGridNode, end: ComputeGridNode) -> Vec<ComputeGridNode> {
		let mut open = OpenList::new(start, end);
		let mut closed = ClosedList::default();
		let mut g_scores = GScores::new(start);

		while let Some(current) = open.pop_lowest_f() {
			if current == end {
				return closed.walk_back_from(&current).collect();
			}

			for neighbor in self.neighbors(&current) {
				if self.obstacles.contains(&neighbor) {
					continue;
				}

				let g = g_scores.get(&current) + Self::distance(&current, &neighbor);

				if g >= g_scores.get(&neighbor) {
					continue;
				}

				open.push(neighbor, g);
				closed.insert(neighbor, current);
				g_scores.insert(neighbor, g);
			}
		}

		vec![]
	}
}

#[derive(Debug, PartialEq, Default)]
struct ClosedList(HashMap<ComputeGridNode, ComputeGridNode>);

impl ClosedList {
	fn insert(&mut self, node: ComputeGridNode, comes_from: ComputeGridNode) {
		self.0.insert(node, comes_from);
	}

	fn walk_back_from<'a>(
		&'a self,
		node: &'a ComputeGridNode,
	) -> impl Iterator<Item = ComputeGridNode> + 'a {
		WalkBack {
			current: Some(node),
			connections: &self.0,
		}
	}
}

struct WalkBack<'a> {
	connections: &'a HashMap<ComputeGridNode, ComputeGridNode>,
	current: Option<&'a ComputeGridNode>,
}

impl Iterator for WalkBack<'_> {
	type Item = ComputeGridNode;

	fn next(&mut self) -> Option<Self::Item> {
		let next = self.current?;
		self.current = self.connections.get(next);

		Some(*next)
	}
}

#[derive(Debug, Default)]
struct OpenList {
	end: ComputeGridNode,
	heap: BinaryHeap<Reverse<Node>>,
}

impl OpenList {
	fn new(start: ComputeGridNode, end: ComputeGridNode) -> Self {
		let f = AStar::distance(&start, &end);
		OpenList {
			end,
			heap: BinaryHeap::from([Reverse(Node { node: start, f })]),
		}
	}

	fn pop_lowest_f(&mut self) -> Option<ComputeGridNode> {
		self.heap.pop().map(|Reverse(Node { node, .. })| node)
	}

	fn push(&mut self, node: ComputeGridNode, g: u32) {
		let f = g + AStar::distance(&node, &self.end);
		self.heap.push(Reverse(Node { node, f }));
	}
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Node {
	node: ComputeGridNode,
	f: u32,
}

impl PartialOrd for Node {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Node {
	fn cmp(&self, other: &Self) -> Ordering {
		self.f
			.cmp(&other.f)
			.then_with(|| self.node.cmp(&other.node))
	}
}

struct GScores(HashMap<ComputeGridNode, u32>);

impl GScores {
	fn new(start: ComputeGridNode) -> Self {
		Self(HashMap::from([(start, 0)]))
	}

	fn insert(&mut self, node: ComputeGridNode, score: u32) {
		self.0.insert(node, score);
	}

	fn get(&self, node: &ComputeGridNode) -> u32 {
		self.0.get(node).cloned().unwrap_or(u32::MAX)
	}
}
