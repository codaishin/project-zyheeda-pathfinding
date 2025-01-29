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

	fn distance(a: ComputeGridNode, b: ComputeGridNode) -> f32 {
		(a.x.abs_diff(b.x) + a.y.abs_diff(b.y)) as f32
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
				return closed.construct_path_from(current).collect();
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

	pub fn construct_path_from(self, node: ComputeGridNode) -> PathIterator {
		PathIterator {
			next: Some(node),
			start: self.start,
			list: self,
		}
	}

	pub fn parent(&self, node: &ComputeGridNode) -> Option<&ComputeGridNode> {
		self.parents.get(node)
	}
}

pub struct PathIterator {
	start: ComputeGridNode,
	list: ClosedList,
	next: Option<ComputeGridNode>,
}

impl PathIterator {
	pub fn remove_redundant_nodes<T>(self, line_of_sight: T) -> CleanedPathIterator<T>
	where
		T: Fn(ComputeGridNode, ComputeGridNode) -> bool,
	{
		CleanedPathIterator {
			los: line_of_sight,
			start: self.start,
			list: self.list,
			next: self.next,
		}
	}
}

impl Iterator for PathIterator {
	type Item = ComputeGridNode;

	fn next(&mut self) -> Option<Self::Item> {
		let current = self.next?;

		self.next = match current == self.start {
			true => None,
			false => self.list.parent(&current).copied(),
		};

		Some(current)
	}
}

pub struct CleanedPathIterator<T>
where
	T: Fn(ComputeGridNode, ComputeGridNode) -> bool,
{
	los: T,
	start: ComputeGridNode,
	list: ClosedList,
	next: Option<ComputeGridNode>,
}

impl<T> Iterator for CleanedPathIterator<T>
where
	T: Fn(ComputeGridNode, ComputeGridNode) -> bool,
{
	type Item = ComputeGridNode;

	fn next(&mut self) -> Option<Self::Item> {
		let current = self.next?;

		self.next = match current == self.start {
			true => None,
			false => {
				let parent = self.list.parent(&current).copied()?;
				let parent_parent = self.list.parent(&parent).copied()?;

				if (self.los)(current, parent_parent) {
					Some(parent_parent)
				} else {
					Some(parent)
				}
			}
		};

		Some(current)
	}
}

pub struct OpenList<'a> {
	end: ComputeGridNode,
	heap: BinaryHeap<Reverse<Node>>,
	dist_f: &'a dyn Fn(ComputeGridNode, ComputeGridNode) -> f32,
}

impl<'a> OpenList<'a> {
	pub fn new(
		start: ComputeGridNode,
		end: ComputeGridNode,
		dist_f: &'a dyn Fn(ComputeGridNode, ComputeGridNode) -> f32,
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

	pub fn push(&mut self, node: ComputeGridNode, g: f32) {
		let f = g + (self.dist_f)(node, self.end);
		self.heap.push(Reverse(Node { node, f }));
	}
}

#[derive(Debug, PartialEq)]
struct Node {
	node: ComputeGridNode,
	f: f32,
}

impl Eq for Node {}

impl PartialOrd for Node {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Node {
	fn cmp(&self, other: &Self) -> Ordering {
		let Some(c_f) = self.f.partial_cmp(&other.f) else {
			panic!(
				"tried to compare {:?} with {:?} (f values are not comparable)",
				self, other
			);
		};
		c_f.then_with(|| self.node.cmp(&other.node))
	}
}

pub struct GScores(HashMap<ComputeGridNode, f32>);

impl GScores {
	pub fn new(start: ComputeGridNode) -> Self {
		Self(HashMap::from([(start, 0.)]))
	}

	pub fn insert(&mut self, node: ComputeGridNode, score: f32) {
		self.0.insert(node, score);
	}

	pub fn get(&self, node: &ComputeGridNode) -> f32 {
		self.0.get(node).cloned().unwrap_or(f32::INFINITY)
	}
}
