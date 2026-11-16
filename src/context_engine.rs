#![doc = include_str!("../docs/CONTEXT.md")]

use crate::{
    graph::{GraphError, KnowledgeGraph, Relation, TraversalDirection, TraversalOptions},
    hybrid::{HybridError, HybridSearchOptions},
    semantic::{EmbeddingError, EmbeddingProvider, SemanticIndex, SemanticSearchOptions},
    text::{is_stopword, normalized_characters, tokenize},
    Context, Engine, SearchOptions,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use unicode_normalization::char::is_combining_mark;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ContextError {
    #[error("this context mode requires a SemanticIndex")]
    MissingSemanticIndex,
    #[error("this context mode requires an embedding provider")]
    MissingProvider,
    #[error(transparent)]
    Embedding(#[from] EmbeddingError),
    #[error(transparent)]
    Hybrid(#[from] HybridError),
    #[error(transparent)]
    Graph(#[from] GraphError),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub enum ContextMode {
    #[default]
    Lexical,
    Semantic,
    Hybrid,
}

#[derive(Debug, Clone)]
pub struct ContextOptions {
    max_nodes: usize,
    max_text_bytes: usize,
    max_item_bytes: usize,
    mode: ContextMode,
    scope: Context,
    lexical: SearchOptions,
    semantic: SemanticSearchOptions,
    hybrid: HybridSearchOptions,
    traversal: TraversalOptions,
    deduplicate: bool,
    include_metadata: bool,
    #[cfg(feature = "provenance")]
    include_sources: bool,
}

impl Default for ContextOptions {
    fn default() -> Self {
        Self {
            max_nodes: 20,
            max_text_bytes: 8192,
            max_item_bytes: 2048,
            mode: ContextMode::Lexical,
            scope: Context::new(),
            lexical: SearchOptions::default(),
            semantic: SemanticSearchOptions::default(),
            hybrid: HybridSearchOptions::default(),
            traversal: TraversalOptions::new(1, 100)
                .with_direction(TraversalDirection::Both)
                .with_start(false),
            deduplicate: true,
            include_metadata: true,
            #[cfg(feature = "provenance")]
            include_sources: true,
        }
    }
}

impl ContextOptions {
    pub fn new(max_nodes: usize, max_text_bytes: usize) -> Self {
        Self {
            max_nodes,
            max_text_bytes,
            ..Self::default()
        }
    }
    pub fn with_item_bytes(mut self, bytes: usize) -> Self {
        self.max_item_bytes = bytes;
        self
    }
    pub fn with_scope(mut self, scope: Context) -> Self {
        self.scope = scope;
        self
    }
    pub fn with_lexical(mut self, options: SearchOptions) -> Self {
        self.lexical = options;
        self.mode = ContextMode::Lexical;
        self
    }
    pub fn with_semantic(mut self, options: SemanticSearchOptions) -> Self {
        self.semantic = options;
        self.mode = ContextMode::Semantic;
        self
    }
    pub fn with_hybrid(mut self, options: HybridSearchOptions) -> Self {
        self.hybrid = options;
        self.mode = ContextMode::Hybrid;
        self
    }
    pub fn with_traversal(mut self, options: TraversalOptions) -> Self {
        self.traversal = options;
        self
    }
    pub fn with_deduplication(mut self, enabled: bool) -> Self {
        self.deduplicate = enabled;
        self
    }
    pub fn with_metadata(mut self, included: bool) -> Self {
        self.include_metadata = included;
        self
    }
    #[cfg(feature = "provenance")]
    pub fn with_source_references(mut self, included: bool) -> Self {
        self.include_sources = included;
        self
    }
    pub fn max_nodes(&self) -> usize {
        self.max_nodes
    }
    pub fn max_text_bytes(&self) -> usize {
        self.max_text_bytes
    }
    pub fn max_item_bytes(&self) -> usize {
        self.max_item_bytes
    }
    pub fn mode(&self) -> ContextMode {
        self.mode
    }
    pub fn scope(&self) -> &Context {
        &self.scope
    }

    fn effective_scope(&self) -> Option<Context> {
        let ranker_scope = match self.mode {
            ContextMode::Lexical => self.lexical.context.as_ref(),
            ContextMode::Semantic => self.semantic.context(),
            ContextMode::Hybrid => self.hybrid.context(),
        };
        ranker_scope.map_or_else(
            || Some(self.scope.clone()),
            |scope| self.scope.intersection(scope),
        )
    }
    fn empty_budget(&self) -> bool {
        self.max_nodes == 0 || self.max_text_bytes == 0 || self.max_item_bytes == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub enum ContextOriginKind {
    Search,
    Provided,
    Graph,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextOrigin<'a> {
    kind: ContextOriginKind,
    mode: Option<ContextMode>,
    rank: Option<usize>,
    score: Option<f64>,
    seed: Option<&'a str>,
    parent: Option<&'a str>,
    relation: Option<&'a Relation>,
    depth: usize,
}

impl<'a> ContextOrigin<'a> {
    fn seed(
        kind: ContextOriginKind,
        mode: Option<ContextMode>,
        rank: usize,
        score: Option<f64>,
    ) -> Self {
        Self {
            kind,
            mode,
            rank: Some(rank),
            score,
            seed: None,
            parent: None,
            relation: None,
            depth: 0,
        }
    }
    pub fn kind(&self) -> ContextOriginKind {
        self.kind
    }
    pub fn mode(&self) -> Option<ContextMode> {
        self.mode
    }
    pub fn rank(&self) -> Option<usize> {
        self.rank
    }
    pub fn score(&self) -> Option<f64> {
        self.score
    }
    pub fn seed_id(&self) -> Option<&'a str> {
        self.seed
    }
    pub fn parent_id(&self) -> Option<&'a str> {
        self.parent
    }
    pub fn relation(&self) -> Option<&'a Relation> {
        self.relation
    }
    pub fn depth(&self) -> usize {
        self.depth
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextReference<'a> {
    id: &'a str,
    metadata: Option<&'a serde_json::Value>,
    origin: ContextOrigin<'a>,
    #[cfg(feature = "provenance")]
    sources: Vec<&'a crate::provenance::SourceReference>,
    #[cfg(feature = "provenance")]
    source_status: crate::provenance::ProvenanceStatus,
}
impl<'a> ContextReference<'a> {
    pub fn id(&self) -> &'a str {
        self.id
    }
    pub fn metadata(&self) -> Option<&'a serde_json::Value> {
        self.metadata
    }
    pub fn origin(&self) -> &ContextOrigin<'a> {
        &self.origin
    }
    #[cfg(feature = "provenance")]
    pub fn sources(&self) -> &[&'a crate::provenance::SourceReference] {
        &self.sources
    }
    #[cfg(feature = "provenance")]
    pub fn provenance_status(&self) -> crate::provenance::ProvenanceStatus {
        self.source_status
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextItem<'a> {
    text: &'a str,
    start_byte: usize,
    end_byte: usize,
    truncated: bool,
    references: Vec<ContextReference<'a>>,
}
impl<'a> ContextItem<'a> {
    pub fn text(&self) -> &'a str {
        self.text
    }
    pub fn start_byte(&self) -> usize {
        self.start_byte
    }
    pub fn end_byte(&self) -> usize {
        self.end_byte
    }
    pub fn truncated(&self) -> bool {
        self.truncated
    }
    pub fn references(&self) -> &[ContextReference<'a>] {
        &self.references
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ContextBundle<'a> {
    items: Vec<ContextItem<'a>>,
    node_count: usize,
    used_text_bytes: usize,
    skipped_missing: usize,
    skipped_scope: usize,
    skipped_budget: usize,
    graph_visits: usize,
    semantic_used: bool,
    fallback_reason: Option<String>,
}
impl<'a> ContextBundle<'a> {
    pub fn items(&self) -> &[ContextItem<'a>] {
        &self.items
    }
    pub fn node_count(&self) -> usize {
        self.node_count
    }
    pub fn used_text_bytes(&self) -> usize {
        self.used_text_bytes
    }
    pub fn skipped_missing(&self) -> usize {
        self.skipped_missing
    }
    pub fn skipped_scope(&self) -> usize {
        self.skipped_scope
    }
    pub fn skipped_budget(&self) -> usize {
        self.skipped_budget
    }
    pub fn graph_visits(&self) -> usize {
        self.graph_visits
    }
    pub fn semantic_used(&self) -> bool {
        self.semantic_used
    }
    pub fn fallback_reason(&self) -> Option<&str> {
        self.fallback_reason.as_deref()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ContextAssembler<'a> {
    lexical: &'a Engine,
    semantic: Option<&'a SemanticIndex>,
    graph: Option<&'a KnowledgeGraph>,
    #[cfg(feature = "provenance")]
    provenance: Option<&'a crate::provenance::ProvenanceStore>,
}
impl<'a> ContextAssembler<'a> {
    pub fn new(index: &'a Engine) -> Self {
        Self {
            lexical: index,
            semantic: None,
            graph: None,
            #[cfg(feature = "provenance")]
            provenance: None,
        }
    }
    pub fn from_semantic(index: &'a SemanticIndex) -> Self {
        Self {
            lexical: index.lexical(),
            semantic: Some(index),
            graph: None,
            #[cfg(feature = "provenance")]
            provenance: None,
        }
    }
    pub fn with_graph(mut self, graph: &'a KnowledgeGraph) -> Self {
        self.graph = Some(graph);
        self
    }
    #[cfg(feature = "provenance")]
    pub fn with_provenance(mut self, provenance: &'a crate::provenance::ProvenanceStore) -> Self {
        self.provenance = Some(provenance);
        self
    }

    pub fn query(
        &self,
        query: &str,
        provider: Option<&mut dyn EmbeddingProvider>,
        options: &ContextOptions,
    ) -> Result<ContextBundle<'a>, ContextError> {
        let Some(scope) = options.effective_scope() else {
            return Ok(ContextBundle::default());
        };
        if options.empty_budget() || query.trim().is_empty() {
            return Ok(ContextBundle::default());
        }
        let mut semantic_used = false;
        let mut fallback_reason = None;
        let seeds: Vec<_> = match options.mode {
            ContextMode::Lexical => {
                let mut search = options.lexical.clone();
                search.context = Some(scope.clone());
                self.lexical
                    .search_with_options(query, &search)
                    .into_iter()
                    .enumerate()
                    .map(|(rank, result)| {
                        (
                            result.entry.id.as_str(),
                            ContextOrigin::seed(
                                ContextOriginKind::Search,
                                Some(options.mode),
                                rank + 1,
                                Some(result.score),
                            ),
                        )
                    })
                    .collect()
            }
            ContextMode::Semantic => {
                let index = self.semantic.ok_or(ContextError::MissingSemanticIndex)?;
                let provider = provider.ok_or(ContextError::MissingProvider)?;
                let search = options.semantic.clone().with_context(scope.clone());
                let results = index.search(query, provider, &search)?;
                semantic_used = !index.is_empty() && search.limit() > 0;
                results
                    .into_iter()
                    .enumerate()
                    .map(|(rank, result)| {
                        (
                            result.entry.id.as_str(),
                            ContextOrigin::seed(
                                ContextOriginKind::Search,
                                Some(options.mode),
                                rank + 1,
                                Some(result.score),
                            ),
                        )
                    })
                    .collect()
            }
            ContextMode::Hybrid => {
                let index = self.semantic.ok_or(ContextError::MissingSemanticIndex)?;
                let provider = provider.ok_or(ContextError::MissingProvider)?;
                let search = options.hybrid.clone().with_context(scope.clone());
                let outcome = index.hybrid_search(query, provider, &search)?;
                semantic_used = outcome.semantic_used();
                fallback_reason = outcome.fallback_reason().map(str::to_owned);
                outcome
                    .into_results()
                    .into_iter()
                    .enumerate()
                    .map(|(rank, result)| {
                        (
                            result.entry.id.as_str(),
                            ContextOrigin::seed(
                                ContextOriginKind::Search,
                                Some(options.mode),
                                rank + 1,
                                Some(result.score),
                            ),
                        )
                    })
                    .collect()
            }
        };
        let mut bundle = self.collect(seeds, query, &scope, options)?;
        bundle.semantic_used = semantic_used;
        bundle.fallback_reason = fallback_reason;
        Ok(bundle)
    }

    pub fn assemble(
        &self,
        ranked_ids: impl IntoIterator<Item = impl AsRef<str>>,
        options: &ContextOptions,
    ) -> Result<ContextBundle<'a>, ContextError> {
        let Some(scope) = options.effective_scope() else {
            return Ok(ContextBundle::default());
        };
        if options.empty_budget() {
            return Ok(ContextBundle::default());
        }
        let mut missing = 0usize;
        let mut excluded = 0usize;
        let seeds = ranked_ids.into_iter().enumerate().filter_map(|(rank, id)| {
            let Some(entry) = self.lexical.get(id.as_ref()) else {
                missing = missing.saturating_add(1);
                return None;
            };
            if !scope.includes(entry) {
                excluded = excluded.saturating_add(1);
                return None;
            }
            Some((
                entry.id.as_str(),
                ContextOrigin::seed(
                    ContextOriginKind::Provided,
                    None,
                    rank.saturating_add(1),
                    None,
                ),
            ))
        });
        let mut bundle = self.collect(seeds, "", &scope, options)?;
        bundle.skipped_missing = missing;
        bundle.skipped_scope = excluded;
        Ok(bundle)
    }

    fn collect(
        &self,
        seeds: impl IntoIterator<Item = (&'a str, ContextOrigin<'a>)>,
        query: &str,
        scope: &Context,
        options: &ContextOptions,
    ) -> Result<ContextBundle<'a>, ContextError> {
        let terms: BTreeSet<_> = tokenize(query)
            .into_iter()
            .filter(|term| !is_stopword(term))
            .collect();
        let mut selection = Selection {
            index: self.lexical,
            #[cfg(feature = "provenance")]
            provenance: self.provenance,
            options,
            scope,
            terms,
            content: BTreeMap::new(),
            seen: BTreeSet::new(),
            bundle: ContextBundle::default(),
        };
        let mut selected_seeds = Vec::new();
        let mut seeds = seeds.into_iter();
        while selection.bundle.node_count < options.max_nodes {
            let Some((id, origin)) = seeds.next() else {
                break;
            };
            if selection.add(id, origin) {
                selected_seeds.push(id);
            }
        }
        if let Some(graph) = self.graph {
            for seed in selected_seeds {
                let remaining = options
                    .traversal
                    .max_nodes()
                    .saturating_sub(selection.bundle.graph_visits);
                if remaining == 0 || selection.bundle.node_count >= options.max_nodes {
                    break;
                }
                if !graph.contains_node(seed) {
                    continue;
                }
                let traversal = options
                    .traversal
                    .clone()
                    .with_start(false)
                    .with_node_limit(remaining);
                let visits = graph.breadth_first_matching(seed, &traversal, |id| {
                    self.lexical
                        .get(id)
                        .is_some_and(|entry| scope.includes(entry))
                })?;
                selection.bundle.graph_visits += visits.len();
                for visit in visits {
                    selection.add(
                        visit.id,
                        ContextOrigin {
                            kind: ContextOriginKind::Graph,
                            mode: None,
                            rank: None,
                            score: None,
                            seed: Some(seed),
                            parent: visit.parent,
                            relation: visit.relation,
                            depth: visit.depth,
                        },
                    );
                }
            }
        }
        Ok(selection.bundle)
    }
}

struct Selection<'a, 'o> {
    index: &'a Engine,
    #[cfg(feature = "provenance")]
    provenance: Option<&'a crate::provenance::ProvenanceStore>,
    options: &'o ContextOptions,
    scope: &'o Context,
    terms: BTreeSet<String>,
    content: BTreeMap<&'a str, usize>,
    seen: BTreeSet<&'a str>,
    bundle: ContextBundle<'a>,
}
impl<'a> Selection<'a, '_> {
    fn add(&mut self, id: &str, origin: ContextOrigin<'a>) -> bool {
        if self.seen.contains(id) {
            return false;
        }
        let Some(entry) = self.index.get(id) else {
            self.bundle.skipped_missing += 1;
            return false;
        };
        if !self.scope.includes(entry) {
            self.bundle.skipped_scope += 1;
            return false;
        }
        if self.bundle.node_count >= self.options.max_nodes {
            self.bundle.skipped_budget += 1;
            return false;
        }
        #[cfg(feature = "provenance")]
        let (sources, source_status) = {
            use crate::provenance::ProvenanceStatus;
            if self.options.include_sources {
                let sources: Vec<_> = self
                    .provenance
                    .into_iter()
                    .flat_map(|store| store.sources_for(&entry.id))
                    .collect();
                let status = if sources.is_empty() {
                    ProvenanceStatus::Missing
                } else {
                    ProvenanceStatus::Referenced
                };
                (sources, status)
            } else {
                (Vec::new(), ProvenanceStatus::Omitted)
            }
        };
        let reference = ContextReference {
            id: &entry.id,
            metadata: self
                .options
                .include_metadata
                .then_some(entry.metadata.as_ref())
                .flatten(),
            origin,
            #[cfg(feature = "provenance")]
            sources,
            #[cfg(feature = "provenance")]
            source_status,
        };
        let normalized = self
            .index
            .normalized_text(id)
            .expect("indexed entry has normalized text");
        if self.options.deduplicate {
            if let Some(&position) = self.content.get(normalized) {
                self.bundle.items[position].references.push(reference);
                self.bundle.node_count += 1;
                self.seen.insert(&entry.id);
                return true;
            }
        }
        let cap = self.options.max_item_bytes.min(
            self.options
                .max_text_bytes
                .saturating_sub(self.bundle.used_text_bytes),
        );
        let (start, end) = excerpt(&entry.text, &self.terms, cap);
        if start == end {
            self.bundle.skipped_budget += 1;
            return false;
        }
        self.bundle.used_text_bytes += end - start;
        self.bundle.node_count += 1;
        self.seen.insert(&entry.id);
        self.content.insert(normalized, self.bundle.items.len());
        self.bundle.items.push(ContextItem {
            text: &entry.text[start..end],
            start_byte: start,
            end_byte: end,
            truncated: start > 0 || end < entry.text.len(),
            references: vec![reference],
        });
        true
    }
}

fn excerpt(text: &str, terms: &BTreeSet<String>, cap: usize) -> (usize, usize) {
    if cap >= text.len() {
        return (0, text.len());
    }
    if cap == 0 {
        return (0, 0);
    }
    let mut anchor = 0;
    let mut word = None;
    for (offset, character) in text
        .char_indices()
        .chain(std::iter::once((text.len(), ' ')))
        .take_while(|_| !terms.is_empty())
    {
        if character.is_alphanumeric() || is_combining_mark(character) {
            word.get_or_insert(offset);
        } else if let Some(start) = word.take() {
            if terms
                .iter()
                .any(|term| normalized_characters(&text[start..offset]).eq(term.chars()))
            {
                anchor = start;
                break;
            }
        }
    }
    let mut start = anchor.saturating_sub(cap / 4);
    while !text.is_char_boundary(start) {
        start += 1;
    }
    let end = floor_boundary(text, start.saturating_add(cap).min(text.len()));
    (start, end)
}

fn floor_boundary(text: &str, mut offset: usize) -> usize {
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}
