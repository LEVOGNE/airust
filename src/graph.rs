#![doc = include_str!("../docs/GRAPH.md")]

use petgraph::{
    algo::toposort,
    graph::DiGraph,
    stable_graph::{EdgeIndex, NodeIndex, StableDiGraph},
    unionfind::UnionFind,
    visit::{EdgeRef, IntoEdgeReferences, NodeIndexable},
    Direction,
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::Arc,
};
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GraphError {
    #[error("{0} must not be empty or whitespace-only")]
    InvalidId(&'static str),
    #[error("unknown graph node: {0}")]
    UnknownNode(String),
    #[error("custom relation type must not be empty")]
    EmptyRelationType,
    #[error("graph index capacity exceeded")]
    Capacity,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[non_exhaustive]
pub enum RelationType {
    ParentChild,
    DependsOn,
    References,
    Related,
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub enum RelationDirection {
    Directed,
    Undirected,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Relation {
    id: Arc<str>,
    source: Arc<str>,
    target: Arc<str>,
    kind: RelationType,
    direction: RelationDirection,
    metadata: Option<serde_json::Value>,
}

impl Relation {
    pub fn new(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        kind: RelationType,
    ) -> Result<Self, GraphError> {
        let relation = Self {
            id: Arc::from(id.into()),
            source: Arc::from(source.into()),
            target: Arc::from(target.into()),
            kind,
            direction: RelationDirection::Directed,
            metadata: None,
        };
        relation.validate()?;
        Ok(relation)
    }

    fn validate(&self) -> Result<(), GraphError> {
        for (field, value) in [
            ("relation ID", self.id()),
            ("source ID", self.source()),
            ("target ID", self.target()),
        ] {
            if value.trim().is_empty() {
                return Err(GraphError::InvalidId(field));
            }
        }
        if matches!(&self.kind,RelationType::Custom(label) if label.trim().is_empty()) {
            return Err(GraphError::EmptyRelationType);
        }
        Ok(())
    }

    pub fn with_direction(mut self, direction: RelationDirection) -> Self {
        self.direction = direction;
        self
    }
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn source(&self) -> &str {
        &self.source
    }
    pub fn target(&self) -> &str {
        &self.target
    }
    pub fn kind(&self) -> &RelationType {
        &self.kind
    }
    pub fn direction(&self) -> RelationDirection {
        self.direction
    }
    pub fn metadata(&self) -> Option<&serde_json::Value> {
        self.metadata.as_ref()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum TraversalDirection {
    #[default]
    Outgoing,
    Incoming,
    Both,
}

#[derive(Debug, Clone)]
pub struct TraversalOptions {
    max_depth: usize,
    max_nodes: usize,
    direction: TraversalDirection,
    allowed_ids: Option<BTreeSet<String>>,
    relation_types: Option<BTreeSet<RelationType>>,
    include_start: bool,
}

impl Default for TraversalOptions {
    fn default() -> Self {
        Self::new(2, 100)
    }
}

impl TraversalOptions {
    pub fn new(max_depth: usize, max_nodes: usize) -> Self {
        Self {
            max_depth,
            max_nodes,
            direction: TraversalDirection::Outgoing,
            allowed_ids: None,
            relation_types: None,
            include_start: true,
        }
    }
    pub fn with_direction(mut self, direction: TraversalDirection) -> Self {
        self.direction = direction;
        self
    }
    pub fn with_allowed_ids(mut self, ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.allowed_ids = Some(ids.into_iter().map(Into::into).collect());
        self
    }
    pub fn with_relation_types(mut self, kinds: impl IntoIterator<Item = RelationType>) -> Self {
        self.relation_types = Some(kinds.into_iter().collect());
        self
    }
    pub fn with_start(mut self, include: bool) -> Self {
        self.include_start = include;
        self
    }
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }
    pub fn max_nodes(&self) -> usize {
        self.max_nodes
    }
    pub fn direction(&self) -> TraversalDirection {
        self.direction
    }
    pub fn include_start(&self) -> bool {
        self.include_start
    }

    fn allows(&self, id: &str) -> bool {
        self.allowed_ids.as_ref().is_none_or(|ids| ids.contains(id))
    }
    fn allows_kind(&self, kind: &RelationType) -> bool {
        self.relation_types
            .as_ref()
            .is_none_or(|kinds| kinds.contains(kind))
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[non_exhaustive]
pub struct Neighbor<'a> {
    pub id: &'a str,
    pub relation: &'a Relation,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[non_exhaustive]
pub struct GraphVisit<'a> {
    pub id: &'a str,
    pub depth: usize,
    pub parent: Option<&'a str>,
    pub relation: Option<&'a Relation>,
}

#[derive(Debug, Clone, Default)]
pub struct KnowledgeGraph {
    graph: StableDiGraph<Arc<str>, Relation>,
    nodes: BTreeMap<Arc<str>, NodeIndex>,
    relations: BTreeMap<Arc<str>, EdgeIndex>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn relation_count(&self) -> usize {
        self.relations.len()
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    pub fn contains_node(&self, id: &str) -> bool {
        self.nodes.contains_key(id)
    }
    pub fn nodes(&self) -> impl Iterator<Item = &str> {
        self.nodes.keys().map(AsRef::as_ref)
    }
    pub fn relations(&self) -> impl Iterator<Item = &Relation> {
        self.relations.values().map(|&index| &self.graph[index])
    }
    pub fn get_relation(&self, id: &str) -> Option<&Relation> {
        self.relations
            .get(id)
            .and_then(|&index| self.graph.edge_weight(index))
    }

    pub fn add_node(&mut self, id: impl Into<String>) -> Result<bool, GraphError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(GraphError::InvalidId("node ID"));
        }
        if self.nodes.contains_key(id.as_str()) {
            return Ok(false);
        }
        let id: Arc<str> = Arc::from(id);
        let index = self
            .graph
            .try_add_node(id.clone())
            .map_err(|_| GraphError::Capacity)?;
        self.nodes.insert(id, index);
        Ok(true)
    }

    pub fn upsert_relation(
        &mut self,
        mut relation: Relation,
    ) -> Result<Option<Relation>, GraphError> {
        relation.validate()?;
        let source = self.node(relation.source())?;
        let target = self.node(relation.target())?;
        relation.source = self.graph[source].clone();
        relation.target = self.graph[target].clone();
        let id = relation.id.clone();
        let index = self
            .graph
            .try_add_edge(source, target, relation)
            .map_err(|_| GraphError::Capacity)?;
        let previous = self.relations.insert(id, index);
        Ok(previous.and_then(|old| self.graph.remove_edge(old)))
    }

    pub fn remove_relation(&mut self, id: &str) -> Option<Relation> {
        self.relations
            .remove(id)
            .and_then(|index| self.graph.remove_edge(index))
    }

    pub fn remove_node(&mut self, id: &str) -> bool {
        let Some(index) = self.nodes.remove(id) else {
            return false;
        };
        let edges: BTreeSet<_> = self
            .graph
            .edges_directed(index, Direction::Outgoing)
            .chain(self.graph.edges_directed(index, Direction::Incoming))
            .map(|edge| edge.id())
            .collect();
        for edge in edges {
            self.relations.remove(self.graph[edge].id());
        }
        self.graph.remove_node(index);
        true
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    fn node(&self, id: &str) -> Result<NodeIndex, GraphError> {
        self.nodes
            .get(id)
            .copied()
            .ok_or_else(|| GraphError::UnknownNode(id.to_owned()))
    }

    fn adjacent(
        &self,
        node: NodeIndex,
        direction: TraversalDirection,
    ) -> impl Iterator<Item = (NodeIndex, &Relation)> {
        self.graph
            .edges_directed(node, Direction::Outgoing)
            .chain(
                self.graph
                    .edges_directed(node, Direction::Incoming)
                    .filter(move |edge| edge.source() != node),
            )
            .filter_map(move |edge| {
                let relation = &self.graph[edge.id()];
                let eligible = relation.direction == RelationDirection::Undirected
                    || match direction {
                        TraversalDirection::Outgoing => edge.source() == node,
                        TraversalDirection::Incoming => edge.target() == node,
                        TraversalDirection::Both => true,
                    };
                eligible.then_some((
                    if edge.source() == node {
                        edge.target()
                    } else {
                        edge.source()
                    },
                    relation,
                ))
            })
    }

    pub fn neighbors(
        &self,
        id: &str,
        direction: TraversalDirection,
    ) -> Result<Vec<Neighbor<'_>>, GraphError> {
        let index = self.node(id)?;
        let mut neighbors: Vec<_> = self
            .adjacent(index, direction)
            .map(|(other, relation)| Neighbor {
                id: &self.graph[other],
                relation,
            })
            .collect();
        neighbors.sort_by(|a, b| {
            a.id.cmp(b.id)
                .then_with(|| a.relation.id().cmp(b.relation.id()))
        });
        Ok(neighbors)
    }

    fn candidates<'a>(
        &'a self,
        node: NodeIndex,
        options: &TraversalOptions,
        seen: &BTreeSet<NodeIndex>,
        budget: usize,
    ) -> Vec<(NodeIndex, &'a Relation)> {
        if budget == 0 {
            return Vec::new();
        }
        let mut selected: BTreeMap<&str, (NodeIndex, &Relation)> = BTreeMap::new();
        for (target, relation) in self.adjacent(node, options.direction) {
            let id = self.graph[target].as_ref();
            if seen.contains(&target)
                || !options.allows(id)
                || !options.allows_kind(relation.kind())
            {
                continue;
            }
            selected
                .entry(id)
                .and_modify(|current| {
                    if relation.id() < current.1.id() {
                        *current = (target, relation);
                    }
                })
                .or_insert((target, relation));
            if selected.len() > budget {
                selected.pop_last();
            }
        }
        selected.into_values().collect()
    }

    fn visit<'a>(&'a self, step: &WalkStep<'a>) -> GraphVisit<'a> {
        GraphVisit {
            id: &self.graph[step.node],
            depth: step.depth,
            parent: step.parent.map(|node| self.graph[node].as_ref()),
            relation: step.relation,
        }
    }

    pub fn breadth_first(
        &self,
        start: &str,
        options: &TraversalOptions,
    ) -> Result<Vec<GraphVisit<'_>>, GraphError> {
        let start = self.node(start)?;
        if options.max_nodes == 0 || !options.allows(&self.graph[start]) {
            return Ok(Vec::new());
        }
        let mut queue = VecDeque::from([WalkStep {
            node: start,
            depth: 0,
            parent: None,
            relation: None,
        }]);
        let mut seen = BTreeSet::from([start]);
        let mut results = Vec::new();
        while let Some(step) = queue.pop_front() {
            if options.include_start || step.depth > 0 {
                results.push(self.visit(&step));
            }
            if results.len() >= options.max_nodes {
                break;
            }
            if step.depth >= options.max_depth {
                continue;
            }
            let budget = options
                .max_nodes
                .saturating_sub(results.len())
                .saturating_sub(queue.len());
            for (node, relation) in self.candidates(step.node, options, &seen, budget) {
                seen.insert(node);
                queue.push_back(WalkStep {
                    node,
                    depth: step.depth + 1,
                    parent: Some(step.node),
                    relation: Some(relation),
                });
            }
        }
        Ok(results)
    }

    pub fn depth_first(
        &self,
        start: &str,
        options: &TraversalOptions,
    ) -> Result<Vec<GraphVisit<'_>>, GraphError> {
        let start = self.node(start)?;
        if options.max_nodes == 0 || !options.allows(&self.graph[start]) {
            return Ok(Vec::new());
        }
        let mut seen = BTreeSet::from([start]);
        let mut results = Vec::new();
        if options.include_start {
            results.push(GraphVisit {
                id: &self.graph[start],
                depth: 0,
                parent: None,
                relation: None,
            });
        }
        if results.len() >= options.max_nodes || options.max_depth == 0 {
            return Ok(results);
        }
        let candidates = self.candidates(start, options, &seen, options.max_nodes - results.len());
        let mut stack = vec![WalkFrame {
            node: start,
            depth: 0,
            neighbors: candidates.into_iter(),
        }];
        while results.len() < options.max_nodes {
            let Some(frame) = stack.last_mut() else {
                break;
            };
            let Some((node, relation)) = frame.neighbors.next() else {
                stack.pop();
                continue;
            };
            let step = WalkStep {
                node,
                depth: frame.depth + 1,
                parent: Some(frame.node),
                relation: Some(relation),
            };
            if !seen.insert(node) {
                continue;
            }
            results.push(self.visit(&step));
            if results.len() < options.max_nodes && step.depth < options.max_depth {
                let neighbors =
                    self.candidates(node, options, &seen, options.max_nodes - results.len());
                if !neighbors.is_empty() {
                    stack.push(WalkFrame {
                        node,
                        depth: step.depth,
                        neighbors: neighbors.into_iter(),
                    });
                }
            }
        }
        Ok(results)
    }

    pub fn has_cycle(&self) -> bool {
        self.cycle_in(None)
    }
    pub fn has_cycle_for(&self, kinds: &[RelationType]) -> bool {
        self.cycle_in(Some(kinds))
    }

    fn cycle_in(&self, kinds: Option<&[RelationType]>) -> bool {
        if self.relations.is_empty() || kinds.is_some_and(|kinds| kinds.is_empty()) {
            return false;
        }
        let accepts =
            |relation: &Relation| kinds.is_none_or(|kinds| kinds.contains(relation.kind()));
        let mut components = UnionFind::<usize>::new(self.graph.node_bound());
        for edge in self.graph.edge_references().filter(|e| {
            accepts(e.weight()) && e.weight().direction == RelationDirection::Undirected
        }) {
            if !components.union(edge.source().index(), edge.target().index()) {
                return true;
            }
        }
        let mut quotient = DiGraph::<(), ()>::new();
        let mut roots = vec![None; self.graph.node_bound()];
        for edge in self
            .graph
            .edge_references()
            .filter(|e| accepts(e.weight()) && e.weight().direction == RelationDirection::Directed)
        {
            let source = components.find_mut(edge.source().index());
            let target = components.find_mut(edge.target().index());
            if source == target {
                return true;
            }
            let source = *roots[source].get_or_insert_with(|| quotient.add_node(()));
            let target = *roots[target].get_or_insert_with(|| quotient.add_node(()));
            quotient.add_edge(source, target, ());
        }
        toposort(&quotient, None).is_err()
    }
}

struct WalkStep<'a> {
    node: NodeIndex,
    depth: usize,
    parent: Option<NodeIndex>,
    relation: Option<&'a Relation>,
}
struct WalkFrame<'a> {
    node: NodeIndex,
    depth: usize,
    neighbors: std::vec::IntoIter<(NodeIndex, &'a Relation)>,
}
