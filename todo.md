# todo

## backlog

- [x] remove hyper
- [x] remove http
- [x] remove logs/tracing — that's the consumer's responsibility
- [ ] split crate into features, put `health` behind one
- [ ] ensure `examples/` is excluded from the published crate
- [ ] set up `cargo publish` pipeline

## ai suggestions

- [ ] add tests for `colon_to_braces` and router param extraction
- [ ] handle query strings in the path parser (strip before matchit lookup)
- [ ] `CHANGELOG.md` diff links at the bottom are stale after 0.1.1 — update when publishing
- [ ] fix pre-existing doctest in `src/status.rs` — `Response::status(204)` never compiled (`u16` not accepted), and `bytes` is undefined in the example
