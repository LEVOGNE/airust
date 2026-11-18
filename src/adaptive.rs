#![doc = include_str!("../docs/ADAPTIVE.md")]
use crate::retrieval::{
    ExcerptAnalyzer, RetrievalBatch, RetrievalError, RetrievalOptions, RetrievalSource,
};
use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SourceRole {
    Lexical,
    Text,
    Semantic,
    Hybrid,
    Custom,
}
#[derive(Debug, Clone)]
pub struct BackendInfo {
    name: String,
    role: SourceRole,
}
impl BackendInfo {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn role(&self) -> SourceRole {
        self.role
    }
}
#[derive(Debug, Clone)]
pub struct RouteChoice {
    name: String,
    reason: String,
}
impl RouteChoice {
    pub fn new(name: impl Into<String>, reason: impl Into<String>) -> Result<Self, RetrievalError> {
        let choice = Self {
            name: name.into(),
            reason: reason.into(),
        };
        if !valid_label(&choice.name, 64) || !valid_label(&choice.reason, 256) {
            return Err(RetrievalError::Configuration("invalid route name/reason"));
        }
        Ok(choice)
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn reason(&self) -> &str {
        &self.reason
    }
}
pub trait RoutingPolicy {
    fn choose(&self, query: &str, available: &[BackendInfo])
        -> Result<RouteChoice, RetrievalError>;
}
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultRoutingPolicy;
impl RoutingPolicy for DefaultRoutingPolicy {
    fn choose(
        &self,
        query: &str,
        available: &[BackendInfo],
    ) -> Result<RouteChoice, RetrievalError> {
        if query.contains('"') {
            let source = available
                .iter()
                .find(|source| source.role == SourceRole::Text)
                .ok_or(RetrievalError::Configuration(
                    "quoted queries require a registered text source",
                ))?;
            return RouteChoice::new(&source.name, "quoted phrase requires positional search");
        }
        let normalized = crate::text::normalize(query);
        let short = crate::text::tokens_normalized(&normalized).take(4).count() <= 3;
        let roles = if short {
            [
                SourceRole::Text,
                SourceRole::Lexical,
                SourceRole::Hybrid,
                SourceRole::Semantic,
                SourceRole::Custom,
            ]
        } else {
            [
                SourceRole::Hybrid,
                SourceRole::Semantic,
                SourceRole::Text,
                SourceRole::Lexical,
                SourceRole::Custom,
            ]
        };
        for role in roles {
            if let Some(source) = available.iter().find(|source| source.role == role) {
                return RouteChoice::new(
                    &source.name,
                    if short {
                        "short query follows the configured role priority"
                    } else {
                        "long query uses the first available configured retrieval role"
                    },
                );
            }
        }
        Err(RetrievalError::Configuration(
            "adaptive retrieval has no registered source",
        ))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RouteState {
    Planned,
    Called,
    Returned,
    Failed,
}
#[derive(Debug, Clone, Serialize)]
pub struct RouteTrace {
    source: String,
    role: SourceRole,
    reason: String,
    state: RouteState,
    recipe: Option<String>,
    received_candidates: Option<usize>,
}
impl RouteTrace {
    pub fn source(&self) -> &str {
        &self.source
    }
    pub fn role(&self) -> SourceRole {
        self.role
    }
    pub fn reason(&self) -> &str {
        &self.reason
    }
    pub fn state(&self) -> RouteState {
        self.state
    }
    pub fn recipe(&self) -> Option<&str> {
        self.recipe.as_deref()
    }
    pub fn received_candidates(&self) -> Option<usize> {
        self.received_candidates
    }
}
struct Backend<'a> {
    source: Box<dyn RetrievalSource + 'a>,
}
pub struct AdaptiveSource<'a> {
    policy: Box<dyn RoutingPolicy + 'a>,
    backends: BTreeMap<String, Backend<'a>>,
    available: Vec<BackendInfo>,
    prepared: Option<(String, String)>,
    analyzer: ExcerptAnalyzer,
    trace: Option<RouteTrace>,
}
impl Default for AdaptiveSource<'_> {
    fn default() -> Self {
        Self::new(DefaultRoutingPolicy)
    }
}
impl<'a> AdaptiveSource<'a> {
    pub fn new(policy: impl RoutingPolicy + 'a) -> Self {
        Self {
            policy: Box::new(policy),
            backends: BTreeMap::new(),
            available: Vec::new(),
            prepared: None,
            analyzer: ExcerptAnalyzer::Legacy,
            trace: None,
        }
    }
    pub fn with_source(
        mut self,
        name: impl Into<String>,
        role: SourceRole,
        source: impl RetrievalSource + 'a,
    ) -> Result<Self, RetrievalError> {
        let name = name.into();
        if !valid_label(&name, 64) {
            return Err(RetrievalError::Configuration("invalid backend name"));
        }
        if self.backends.len() >= 16 {
            return Err(RetrievalError::Configuration(
                "adaptive backend count exceeds sixteen",
            ));
        }
        if self.backends.contains_key(&name) {
            return Err(RetrievalError::Configuration(
                "duplicate adaptive backend name",
            ));
        }
        self.available.push(BackendInfo {
            name: name.clone(),
            role,
        });
        self.backends.insert(
            name,
            Backend {
                source: Box::new(source),
            },
        );
        self.prepared = None;
        Ok(self)
    }
    pub fn last_route(&self) -> Option<&RouteTrace> {
        self.trace.as_ref()
    }
    pub fn available(&self) -> &[BackendInfo] {
        &self.available
    }
    fn plan(&mut self, query: &str, options: &RetrievalOptions) -> Result<(), RetrievalError> {
        self.prepared = None;
        self.trace = None;
        self.analyzer = ExcerptAnalyzer::Legacy;
        if query.trim().is_empty() {
            return Err(RetrievalError::Configuration("query is blank"));
        }
        if query.len() > 16384usize.min(options.max_query_bytes()) {
            return Err(RetrievalError::Budget("adaptive query bytes"));
        }
        #[cfg(feature = "execution")]
        if let Some(control) = options.control() {
            control.checkpoint()?;
            control.progress(crate::execution::WorkPhase::Routing, 0, Some(1));
            control.checkpoint()?;
        }
        let choice = self.policy.choose(query, &self.available)?;
        let info = self
            .available
            .iter()
            .find(|source| source.name == choice.name)
            .ok_or(RetrievalError::Configuration(
                "policy selected an unregistered backend",
            ))?;
        let backend = self
            .backends
            .get_mut(&choice.name)
            .ok_or(RetrievalError::Configuration("backend registry invariant"))?;
        self.trace = Some(RouteTrace {
            source: choice.name.clone(),
            role: info.role,
            reason: choice.reason,
            state: RouteState::Planned,
            recipe: None,
            received_candidates: None,
        });
        if let Err(error) = backend.source.prepare_query(query, options) {
            if let Some(trace) = &mut self.trace {
                trace.state = RouteState::Failed;
            }
            return Err(error);
        }
        self.analyzer = backend.source.excerpt_analyzer();
        self.prepared = Some((query.to_owned(), choice.name));
        #[cfg(feature = "execution")]
        if let Some(control) = options.control() {
            control.progress(crate::execution::WorkPhase::Routing, 1, Some(1));
            control.checkpoint()?;
        }
        Ok(())
    }
}
impl RetrievalSource for AdaptiveSource<'_> {
    fn prepare_query(
        &mut self,
        query: &str,
        options: &RetrievalOptions,
    ) -> Result<(), RetrievalError> {
        self.plan(query, options)
    }
    fn excerpt_analyzer(&self) -> ExcerptAnalyzer {
        self.analyzer.clone()
    }
    fn retrieve<'b>(
        &'b mut self,
        query: &str,
        options: &RetrievalOptions,
    ) -> Result<RetrievalBatch<'b>, RetrievalError> {
        if !self
            .prepared
            .as_ref()
            .is_some_and(|(prepared, _)| prepared == query)
        {
            self.plan(query, options)?;
        }
        let (_, name) = self
            .prepared
            .take()
            .ok_or(RetrievalError::Configuration("missing prepared route"))?;
        #[cfg(feature = "execution")]
        if let Some(control) = options.control() {
            control.checkpoint()?;
        }
        if let Some(trace) = &mut self.trace {
            trace.state = RouteState::Called;
        }
        let result = self
            .backends
            .get_mut(&name)
            .ok_or(RetrievalError::Configuration("backend registry invariant"))?
            .source
            .retrieve(query, options);
        #[cfg(feature = "execution")]
        if let Some(control) = options.control() {
            if let Err(error) = control.checkpoint() {
                if let Some(trace) = &mut self.trace {
                    trace.state = RouteState::Failed;
                }
                return Err(error.into());
            }
        }
        match result {
            Ok(batch) => {
                if batch.recipe.trim().is_empty()
                    || batch.recipe.len() > 1024
                    || batch.candidates.len() > options.candidates()
                {
                    if let Some(trace) = &mut self.trace {
                        trace.state = RouteState::Failed;
                    }
                    return Err(RetrievalError::Output("invalid routed batch recipe/count"));
                }
                if let Some(trace) = &mut self.trace {
                    trace.state = RouteState::Returned;
                    trace.recipe = Some(batch.recipe.clone());
                    trace.received_candidates = Some(batch.candidates.len());
                }
                Ok(batch)
            }
            Err(error) => {
                if let Some(trace) = &mut self.trace {
                    trace.state = RouteState::Failed;
                }
                Err(error)
            }
        }
    }
}
fn valid_label(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}
