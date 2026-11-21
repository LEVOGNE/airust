#![cfg(feature = "execution")]

use airust::{
    execution::{CancellationToken, ExecutionError, WorkControl, WorkPhase},
    index::ControlledIndexError,
    matching::{MatchOptions, MatchStrategy},
    Engine, KnowledgeEntry,
};
use std::time::Duration;

#[test]
fn controlled_matching_preserves_all_scores_and_explanations() {
    let texts = [
        "Rust ownership",
        "rust ownership!",
        "Rust memory",
        "Straße",
        "STRASSE",
        "Grüße 🌍",
        "東京 Rust",
        "á",
        "á",
        "",
        "abc",
        "abdc",
    ];
    let mut index = Engine::from_entries(
        texts
            .iter()
            .enumerate()
            .filter(|(_, text)| !text.is_empty())
            .map(|(i, text)| KnowledgeEntry::new(i.to_string(), *text)),
    )
    .unwrap();
    index
        .upsert(KnowledgeEntry::new("2", "Rust borrowing"))
        .unwrap();
    index.remove("3");
    for query in texts.iter().chain(["strase", "Rust", "a", "Ω"].iter()) {
        for strategy in [
            MatchStrategy::Exact,
            MatchStrategy::Fuzzy,
            MatchStrategy::TokenOverlap,
        ] {
            for threshold in [0.0, 0.25, 0.5, 2.0 / 3.0, 0.9, 1.0] {
                for limit in [0, 1, 20] {
                    let options = MatchOptions {
                        limit,
                        min_similarity: threshold,
                        strategy,
                    };
                    let expected = index.match_with_options(query, &options).unwrap();
                    let actual = index
                        .match_controlled(query, &options, &WorkControl::default())
                        .unwrap();
                    let wire = |hits: &[airust::matching::MatchResult<'_>]| {
                        hits.iter()
                            .map(|h| {
                                (
                                    h.entry.id.clone(),
                                    h.score.to_bits(),
                                    h.similarity.exact,
                                    h.similarity.character.to_bits(),
                                    h.similarity.token.to_bits(),
                                )
                            })
                            .collect::<Vec<_>>()
                    };
                    assert_eq!(
                        wire(&actual),
                        wire(&expected),
                        "{query:?} {strategy:?} {threshold} {limit}"
                    );
                }
            }
        }
        for entry in index.entries() {
            assert_eq!(
                serde_json::to_value(
                    index
                        .relate_controlled(&entry.id, 20, &WorkControl::default())
                        .unwrap()
                )
                .unwrap(),
                serde_json::to_value(index.relate(&entry.id, 20).unwrap()).unwrap()
            );
        }
    }
}

#[test]
fn cancellation_and_deadlines_cover_matching_and_relations() {
    let index = Engine::from_entries([
        KnowledgeEntry::new("a", "Rust ownership"),
        KnowledgeEntry::new("b", "Rust memory"),
    ])
    .unwrap();
    let token = CancellationToken::new();
    token.cancel();
    let control = WorkControl::new(token);
    assert!(matches!(
        index.match_controlled("Rust", &MatchOptions::default(), &control),
        Err(ControlledIndexError::Execution(ExecutionError::Cancelled))
    ));
    assert!(matches!(
        index.relate_controlled("a", 2, &control),
        Err(ControlledIndexError::Execution(ExecutionError::Cancelled))
    ));
    let expired = WorkControl::default().with_timeout(Duration::ZERO).unwrap();
    assert!(matches!(
        index.match_controlled("Rust", &MatchOptions::default(), &expired),
        Err(ControlledIndexError::Execution(
            ExecutionError::DeadlineExceeded
        ))
    ));
    assert!(index
        .relate_controlled("missing", 2, &WorkControl::default())
        .is_err());
    assert!(index
        .match_controlled(
            "Rust",
            &MatchOptions {
                min_similarity: f64::NAN,
                ..Default::default()
            },
            &WorkControl::default()
        )
        .is_err());
}

#[test]
fn observer_cancellation_stops_candidate_ranking() {
    let index = Engine::from_entries(
        (0..1000).map(|i| KnowledgeEntry::new(i.to_string(), format!("Rust memory {i}"))),
    )
    .unwrap();
    for relate in [false, true] {
        let token = CancellationToken::new();
        let signal = token.clone();
        let control = WorkControl::new(token).with_observer(
            move |event: &airust::execution::ProgressEvent| {
                if event.phase() == WorkPhase::Ranking {
                    signal.cancel();
                }
            },
        );
        let result = if relate {
            index.relate_controlled("0", 5, &control).map(|_| ())
        } else {
            index
                .match_controlled("Rust", &MatchOptions::default(), &control)
                .map(|_| ())
        };
        assert!(matches!(
            result,
            Err(ControlledIndexError::Execution(ExecutionError::Cancelled))
        ));
    }
}
