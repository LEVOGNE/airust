#![cfg(all(
    feature = "retrieval",
    feature = "advanced-search",
    feature = "documents",
    feature = "shared-index",
    feature = "execution",
    feature = "cache"
))]

#[path = "support/fuzz_lexical.rs"]
mod fuzz_lexical;

#[path = "support/fuzz_manifest.rs"]
mod fuzz_manifest;

#[test]
fn versioned_lexical_seeds_match_the_manifest_and_replay() {
    fuzz_manifest::SeedManifest::replay("lexical", |bytes| {
        fuzz_lexical::LexicalCampaign::exercise(bytes);
    });
}
