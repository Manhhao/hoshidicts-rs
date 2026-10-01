# hoshidicts-rs

Rust bindings for [hoshidicts](https://github.com/Manhhao/hoshidicts), a

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

## License

GPL-3.0-or-later
