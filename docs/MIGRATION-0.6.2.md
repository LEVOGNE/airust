# Upgrade to the 0.6.2 development candidate

This checkout is unreleased. Published `airust = "=0.6.1"` consumers retain their current behavior; no registry 0.6.2 package is asserted to exist.

No existing public method, default feature, model recipe, knowledge/database/document/workspace format or search default changes. Version-qualified MCP schema identifiers advance with the crate version; schema shapes and tool names remain compatible. Backend 0.3.5 remains unchanged. Core/provider-neutral builds retain Rust 1.89; local ONNX adapters require Rust 1.92.

For applications that repeatedly unload/reload MiniLM and need the native heap lifetime to end, build the matching trusted `airust-model-worker` from this checkout with `--no-default-features --features local-reranking`. Replace only the explicitly chosen scorer with `ProcessReranker::multilingual_minilm_v2_from_snapshot`. Supply the worker executable, existing pinned model directory, trusted runtime, 1..64 CPU threads and an existing application-owned snapshot parent. Keep enough disk space for the verified 470883696-byte model copy plus filesystem overhead per active worker. Use the unchanged `Reranker`/`rerank` APIs.

In the reference pilot, set `reranker.isolated` to true and supply `model_snapshot_directory`. The pilot binary handles its own reserved worker role. Omitted/false preserves direct loading. The profile format remains additive; older binaries reject the new field rather than silently adopting process ownership. `profile-check` stays offline and does not verify assets or executable permissions.

Stop/recreate an unhealthy scorer explicitly. Initialization publishes no scorer on failure. Invalid caller input does not poison a healthy worker; malformed worker output, crashes and timeouts stop it. `Drop` closes healthy input, permits normal teardown, then kills/reaps if needed before deleting only the private snapshot subtree. Original models, databases and user data remain caller-owned. An abrupt application exit can leave temporary files. Trusted workers must not leave descendants retaining the pipes. This is not a security sandbox or hard OS resource quota.

Read [the complete process contract](LOCAL-MODEL-PROCESSES.md) and [the measurements, negative experiments and latency limits](measurements/native-lifecycle-0.6.2/README.md). The old direct constructors still have the measured allocator-retention behavior. Actual E5 inference, Windows/Intel execution, low-RAM device behavior and independent customer quality are not established by this change.
