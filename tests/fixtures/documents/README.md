# Owned document format fixtures

These synthetic UTF-8 samples contain no imported book text. `snapshot-v1.json` and `identity-v1.json` were calculated independently with Python's standard `hashlib` and little-endian unsigned 64-bit framing. Rust tests must reproduce both the document revision and the first of two repeated Rust passages.

Every field is part of the documented version-one contract. Original source bytes are the ASCII string `source:001`. SHA-256 strings use lowercase hexadecimal. A frame is the eight-byte little-endian length followed by that field's bytes. See the public document guide for the complete field order.
