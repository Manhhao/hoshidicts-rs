# hoshidicts-rs

Safe Rust bindings for [hoshidicts](https://github.com/Manhhao/hoshidicts), a
Yomitan-dictionary import and lookup engine.

## Building

The C++ engine is a submodule and needs a C++23 toolchain (GCC 14+, Clang 17+,
or MSVC 2022). Check it out recursively:

```sh
git submodule update --init --recursive
cargo build
```

On Linux, `build.rs` links the C++ standard library; point Cargo's linker at the
same C++ compiler used to build the engine (for example `g++-14`) so the
libstdc++ versions match.

## Usage

```rust
use hoshidicts::{Deinflector, LookupFrequencyOrder, LookupOptions, OwnedLookup, Query};

let mut query = Query::new();
query.add_term_dict("jitendex")?;
query.add_freq_dict("BCCWJ")?;

let lookup = OwnedLookup::new(query, Deinflector::new());

let options = LookupOptions {
    frequency_dictionary: Some("BCCWJ"),
    frequency_order: LookupFrequencyOrder::Ascending,
    primary_reading: None,
};
let results = lookup.run_with_options("蜂が好きです", 32, 16, &options)?;
for result in results.results() {
    println!("{}", result.term().expression());
}
```

`run` keeps its default-options behavior; `run_with_options` adds the
frequency-ordering and primary-reading controls.

## Containers

A `.hoshi` container packs an imported dictionary directory into one file that
the engine can query directly, without unpacking it first.

```rust
use hoshidicts::{index, pack, verify, Query};

let bytes = pack("jitendex", "jitendex.hoshi")?;
let payload_version = verify("jitendex.hoshi")?;
let summary = index("jitendex.hoshi")?;

let mut query = Query::new();
query.add_term_dict("jitendex.hoshi")?;
```

`pack` writes the container, verifies it before moving it into place, and
returns its size in bytes. `verify` checks every section against its checksum
and returns the payload version. `index` returns the summary an import wrote as
JSON — the title, revision and entry counts — enough to identify a container and
tell a term dictionary from a kanji or IPA one without loading it. `add_term_dict`
and its siblings take either a directory or a container path.

## Threading

`Query`, `Deinflector`, `Lookup`, and `OwnedLookup` are `Send` but not `Sync`:
an owner may move to another thread, but concurrent calls must be serialized by
the caller. Result views borrow their owner and must be consumed before it is
dropped or moved.

## License

GPL-3.0-or-later, matching the engine.
