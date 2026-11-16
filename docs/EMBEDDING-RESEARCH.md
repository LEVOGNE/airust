# Embedding provider selection

Checked on November 16, 2026 against registry metadata, upstream source, and the model card. The current FastEmbed patch selected for testing is 7.7.1. It is Apache-2.0-licensed. Its ONNX path uses `ort 2.0.0-rc.15`, whose declared minimum Rust version is 1.92, despite FastEmbed's own lower package declaration. The feature-specific toolchain requirement is therefore tested at 1.92.

FastEmbed supports application-supplied model bytes and tokenizer files. This avoids its default model-hub downloads. AIRust enables only dynamic ONNX loading, uses CPU execution, and leaves hub and binary-download features off. The provider-neutral feature has no new dependency; native inference is a separately enabled adapter. This preserves a small default library while reusing maintained inference and tokenization code.

The evaluated E5 checkpoint is `intfloat/multilingual-e5-small`, revision `614241f622f53c4eeff9890bdc4f31cfecc418b3`. Its model card specifies multilingual retrieval, 384 output dimensions, appropriate query/document prefixes, and an MIT license. The actual checked ONNX file contains 470,268,510 bytes; the tokenizer contains another 17,082,730 bytes. These sizes were verified from downloaded artifacts and recorded with hashes.

This is a relatively small multilingual embedding model, but its unquantized assets are still hundreds of megabytes. It belongs outside the library package. Quantized alternatives may reduce footprint but need separate pooling, compatibility, and quality checks. This release does not silently substitute another checkpoint.

The chosen adapter uses mean pooling, a 512-token limit, E5 prefixes, and two CPU threads in the evaluation. The model recipe hashes actual model and tokenizer bytes. A separate ONNX Runtime 1.32.0 ARM64 library was obtained from Microsoft's release. Runtime and model files are not fetched by Cargo, the library constructor, or normal tests.

The small public reference corpus supplies fixed German and multilingual smoke cases. Its results establish those cases only. Benchmarks on labeled application corpora and larger independent retrieval datasets remain necessary before production quality claims.

Primary references:

- [FastEmbed releases and source](https://github.com/Anush008/fastembed-rs).
- [FastEmbed 7.7.1 registry metadata](https://crates.io/crates/fastembed/7.7.1).
- [ORT Rust bindings](https://github.com/pykeio/ort).
- [E5 model card and license](https://huggingface.co/intfloat/multilingual-e5-small).
- [Pinned model files](https://huggingface.co/intfloat/multilingual-e5-small/tree/614241f622f53c4eeff9890bdc4f31cfecc418b3).
- [ONNX Runtime 1.32.0](https://github.com/microsoft/onnxruntime/releases/tag/v1.32.0).
- [Cargo API compatibility guidance](https://doc.rust-lang.org/cargo/reference/semver.html).
