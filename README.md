# Zakura Arti

[Arti](https://arti.torproject.org/) is the Tor Project’s implementation of
[Tor](https://www.torproject.org/) in Rust. It provides an embedded client that
applications can use to route connections through the Tor network.

This is a small, maintained fork for wallet applications. Its scope is the API
surface and dependencies needed to embed Arti in wallets, with focused cleanup
and fixes for supply-chain and security issues. Changes stay narrowly scoped
so we can continue incorporating upstream improvements.

The [upstream Arti repository](https://gitlab.torproject.org/tpo/core/arti)
remains the source project. Upstream history and attribution are preserved.
The Rust code retains its [MIT](LICENSE-MIT) / [Apache 2.0](LICENSE-APACHE)
licensing; individual files and optional components retain their original
license notices.
