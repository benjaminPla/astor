# todo

## backlog

- [ ] set up `cargo publish` pipeline
- [ ] remove all `unwrap()` and others similar, standarize an error
- [ ] double check if lowecase methods can bypass ngnix and handle it (better in the ngnix conf, not astor)
- [x] bug getting more than one query param, I think I need to store them in a hasmap again, because it can be multiple, also Option maybe
- [x] remove hyper
- [x] remove http
- [x] remove logs/tracing — that's the consumer's responsibility
- [x] clean README.md file (philosophy, reverse-proxy, etc) and refer to the docs - mainteining both is a mess
- [x] ensure `examples/` is excluded from the published crate
