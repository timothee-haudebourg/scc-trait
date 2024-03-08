use std::collections::{HashMap, HashSet};

use super::{Scc, Components};

// Solve dependencies using Tarjan's SCC algorithm.
struct Data {
	index: u32,
	lowlink: u32,
	on_stack: bool,
	component: usize,
}

pub fn scc<G: ?Sized + Scc>(graph: &G) -> Components<G::Vertex> {
	let mut map: HashMap<G::Vertex, Data> = HashMap::new();
	let mut stack = Vec::new();
	let mut components = Vec::new();

	for v in graph.vertices() {
		if !map.contains_key(&v) {
			strong_connect(graph, v, &mut stack, &mut map, &mut components);
		}
	}

	let vertex_to_component: HashMap<_, _> = map
			.into_iter()
			.map(|(v, data)| (v, data.component))
			.collect();

	let successors: Vec<HashSet<_>> = components
		.iter()
		.map(|component| {
			component
				.iter()
				.flat_map(|v| {
					graph.successors(*v)
						.into_iter()
						.map(|sc| *vertex_to_component.get(&sc).unwrap())
				})
				.collect()
		})
		.collect();

	Components {
		vertex_to_component,
		list: components,
		successors,
	}
}

fn strong_connect<G: ?Sized + Scc>(
	graph: &G,
	v: G::Vertex,
	stack: &mut Vec<G::Vertex>,
	map: &mut HashMap<G::Vertex, Data>,
	components: &mut Vec<Vec<G::Vertex>>,
) -> u32 {
	let index = map.len() as u32;
	stack.push(v);
	map.insert(
		v,
		Data {
			index,
			lowlink: index,
			on_stack: true,
			component: 0,
		},
	);

	// Consider successors of v
	for w in graph.successors(v) {
		let new_v_lowlink = match map.get(&w) {
			None => {
				// Successor w has not yet been visited; recurse on it
				let w_lowlink = strong_connect(graph, w, stack, map, components);
				Some(std::cmp::min(map[&v].lowlink, w_lowlink))
			}
			Some(w_data) => {
				if w_data.on_stack {
					// Successor w is in stack S and hence in the current SCC
					// If w is not on stack, then (v, w) is an edge pointing to an SCC already found and must be ignored
					// Note: The next line may look odd - but is correct.
					// It says w.index not w.lowlink; that is deliberate and from the original paper
					Some(std::cmp::min(map[&v].lowlink, w_data.index))
				} else {
					None
				}
			}
		};

		if let Some(new_v_lowlink) = new_v_lowlink {
			map.get_mut(&v).unwrap().lowlink = new_v_lowlink;
		}
	}

	let lowlink = map[&v].lowlink;

	// If v is a root node, pop the stack and generate an SCC
	if lowlink == map[&v].index {
		// Start a new strongly connected component
		let mut component = Vec::new();

		loop {
			let w = stack.pop().unwrap();
			let w_data = map.get_mut(&w).unwrap();
			w_data.on_stack = false;
			w_data.component = components.len();

			// Add w to current strongly connected component
			component.push(w);

			if w == v {
				break;
			}
		}

		// Output the current strongly connected component
		components.push(component)
	}

	lowlink
}
