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

	fn distance(a: ComputeGridNode, b: ComputeGridNode) -> u32 {
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
		let mut open = OpenList::new(start, end, &Self::distance);
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

				let g = g_scores.get(&current) + Self::distance(current, neighbor);

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

#[derive(Debug, Default)]
pub struct ClosedList {
	start: ComputeGridNode,
	parents: HashMap<ComputeGridNode, ComputeGridNode>,
}

impl ClosedList {
	pub fn new(start: ComputeGridNode) -> Self {
		Self {
			start,
			parents: HashMap::from([(start, start)]),
		}
	}

	pub fn insert(&mut self, node: ComputeGridNode, comes_from: ComputeGridNode) {
		self.parents.insert(node, comes_from);
	}

	pub fn walk_back_from<'a>(
		&'a self,
		node: &'a ComputeGridNode,
	) -> impl Iterator<Item = ComputeGridNode> + 'a {
		WalkBack {
			current: Some(node),
			start: self.start,
			parents: &self.parents,
		}
	}

	pub fn parent(&self, node: &ComputeGridNode) -> Option<&ComputeGridNode> {
		self.parents.get(node)
	}
}

struct WalkBack<'a> {
	start: ComputeGridNode,
	parents: &'a HashMap<ComputeGridNode, ComputeGridNode>,
	current: Option<&'a ComputeGridNode>,
}

impl Iterator for WalkBack<'_> {
	type Item = ComputeGridNode;

	fn next(&mut self) -> Option<Self::Item> {
		let next = self.current?;

		self.current = match next == &self.start {
			true => None,
			false => self.parents.get(next),
		};

		Some(*next)
	}
}

pub struct OpenList<'a> {
	end: ComputeGridNode,
	heap: BinaryHeap<Reverse<Node>>,
	dist_f: &'a dyn Fn(ComputeGridNode, ComputeGridNode) -> u32,
}

impl<'a> OpenList<'a> {
	pub fn new(
		start: ComputeGridNode,
		end: ComputeGridNode,
		dist_f: &'a dyn Fn(ComputeGridNode, ComputeGridNode) -> u32,
	) -> Self {
		let f = dist_f(start, end);
		OpenList {
			end,
			dist_f,
			heap: BinaryHeap::from([Reverse(Node { node: start, f })]),
		}
	}

	pub fn pop_lowest_f(&mut self) -> Option<ComputeGridNode> {
		self.heap.pop().map(|Reverse(Node { node, .. })| node)
	}

	pub fn push(&mut self, node: ComputeGridNode, g: u32) {
		let f = g + (self.dist_f)(node, self.end);
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

pub struct GScores(HashMap<ComputeGridNode, u32>);

impl GScores {
	pub fn new(start: ComputeGridNode) -> Self {
		Self(HashMap::from([(start, 0)]))
	}

	pub fn insert(&mut self, node: ComputeGridNode, score: u32) {
		self.0.insert(node, score);
	}

	pub fn get(&self, node: &ComputeGridNode) -> u32 {
		self.0.get(node).cloned().unwrap_or(u32::MAX)
	}
}
