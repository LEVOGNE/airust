# Migrating to AIRust 0.2.19

This patch adds development stress examples, regression tests and measured documentation. It changes no public runtime method, dependency, default feature, persisted format or response JSON. Existing consumers can update their exact version pin without an API rewrite.

Large workloads are explicit and count-guarded; normal CI uses 1,000-record smoke cases. Memory depends on record size, tokens, retained copies and concurrency. Do not infer production capacity or native-model performance from the synthetic lexical evidence. See [methodology](STRESS.md). No DenkMaps/MCP integration is performed.
