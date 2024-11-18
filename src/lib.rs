use std::{
	collections::{HashMap, HashSet},
	hash::Hash,
};

mod tarjan;

/// Graph on which strongly connected components can be computed.
pub trait Scc {
	/// Graph vertex reference type.
	type Vertex: Copy + Eq + Hash;

	/// Returns an iterator over the vertices of the graph.
	fn vertices(&self) -> impl '_ + IntoIterator<Item = Self::Vertex>;

	/// Returns an iterator over the successors of the given vertex.
	fn successors(&self, v: Self::Vertex) -> impl '_ + IntoIterator<Item = Self::Vertex>;

	/// Computes the strongly connected components of the graph.
	fn strongly_connected_components(&self) -> Components<Self::Vertex> {
		tarjan::scc(self)
	}
}

/// Strongly connected components.
pub struct Components<V> {
	/// Components list.
	list: Vec<Vec<V>>,

	/// Map from vertices to component index.
	vertex_to_component: HashMap<V, usize>,

	/// Component successors.
	successors: Vec<HashSet<usize>>,
}

impl<V> Components<V> {
	/// Returns the number of strongly connected components.
	pub fn len(&self) -> usize {
		self.list.len()
	}

	/// Checks if there are no components.
	pub fn is_empty(&self) -> bool {
		self.list.is_empty()
	}

	/// Returns an iterator over the strongly connected components.
	pub fn iter(&self) -> impl Iterator<Item = &[V]> {
		self.list.iter().map(Vec::as_slice)
	}

	/// Returns the index of the given vertex's strongly connected component.
	pub fn vertex_component_index(&self, v: &V) -> Option<usize>
	where
		V: Eq + Hash,
	{
		self.vertex_to_component.get(v).cloned()
	}

	/// Returns the component with the given index `i`.
	pub fn get_by_index(&self, i: usize) -> Option<&[V]> {
		self.list.get(i).map(Vec::as_slice)
	}

	/// Return the given vertex's strongly connected component.
	pub fn get(&self, v: &V) -> Option<&[V]>
	where
		V: Eq + Hash
	{
		self.get_by_index(self.vertex_component_index(v)?)
	}

	pub fn successors(&self, i: usize) -> Option<impl '_ + Iterator<Item = usize>> {
		self.successors.get(i).map(|s| s.iter().cloned())
	}

	pub fn is_cyclic(&self, i: usize) -> bool {
		self.successors.get(i).unwrap().contains(&i)
	}

	fn remove_indirect_successors(&self, result: &mut HashSet<usize>, i: usize) {
		for j in self.successors(i).unwrap() {
			result.remove(&j);
			self.remove_indirect_successors(result, j);
		}
	}

	pub fn direct_successors(&self, i: usize) -> Option<HashSet<usize>> {
		let mut result: HashSet<_> = self.successors(i)?.collect();

		for j in self.successors(i).unwrap() {
			self.remove_indirect_successors(&mut result, j);
		}

		Some(result)
	}

	/// Returns the depth of each component.
	///
	/// The depth of a component is the maximum of the depth of its predecessors
	/// plus 1. A component with no predecessors has depth 0.
	pub fn depths(&self) -> Vec<usize> {
		let mut depth = vec![0; self.list.len()];
		let mut stack: Vec<_> = depth.iter().cloned().enumerate().collect();

		while let Some((i, new_depth)) = stack.pop() {
			if depth[i] == 0 || new_depth > depth[i] {
				depth[i] = new_depth;
				for c in self.successors(i).unwrap() {
					if c != i {
						stack.push((c, new_depth + 1))
					}
				}
			}
		}

		depth
	}

	pub fn predecessors(&self) -> Vec<HashSet<usize>> {
		let mut predecessors = Vec::new();
		predecessors.resize_with(self.list.len(), HashSet::default);

		for (i, successors) in self.successors.iter().enumerate() {
			for &j in successors {
				predecessors[j].insert(i);
			}
		}

		predecessors
	}

	/// Order components by depth.
	///
	/// The depth of a component is the maximum of the depth of its predecessors
	/// plus 1. A component with no predecessors has depth 0.
	pub fn order_by_depth(&self) -> Vec<usize> {
		let depth = self.depths();
		let mut ordered_components: Vec<_> = (0..self.list.len()).collect();
		ordered_components.sort_unstable_by_key(|i| depth[*i]);
		ordered_components
	}
}

/// Returns the depth of each component.
///
/// The depth of a component is the maximum of the depth of its predecessors
/// plus 1. A component with no predecessors has depth 0.
pub fn depths(predecessors: &[HashSet<usize>]) -> Vec<usize> {
	let mut depth = vec![0; predecessors.len()];
	let mut stack: Vec<_> = depth.iter().cloned().enumerate().collect();

	while let Some((i, new_depth)) = stack.pop() {
		if depth[i] == 0 || new_depth > depth[i] {
			depth[i] = new_depth;
			for &c in &predecessors[i] {
				if c != i {
					stack.push((c, new_depth + 1))
				}
			}
		}
	}

	depth
}

impl Scc for Vec<HashSet<usize>> {
	type Vertex = usize;

	fn vertices(&self) -> impl '_ + IntoIterator<Item = Self::Vertex> {
		0..self.len()
	}

	fn successors(&self, v: Self::Vertex) -> impl '_ + IntoIterator<Item = Self::Vertex> {
		self[v].iter().copied()
	}
}

impl<T: Copy + Eq + Hash> Scc for HashMap<T, HashSet<T>> {
	type Vertex = T;

	fn vertices(&self) -> impl '_ + IntoIterator<Item = Self::Vertex> {
		self.keys().copied()
	}

	fn successors(&self, v: Self::Vertex) -> impl '_ + IntoIterator<Item = Self::Vertex> {
		self[&v].iter().copied()
	}
}