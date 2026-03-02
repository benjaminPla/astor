# Contributing to astor

astor exists for one reason: Rust services behind a reverse proxy. Not Rust services that *are* a reverse proxy.

If your idea belongs in nginx config, it does not belong here.
If your idea makes astor better at being astor, welcome.

---

- Fork the repo, branch from `master`, open a PR against `master`.
- PR title: `<type>: <short description>` — types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`.
- `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check` must all pass.
- Add or update tests for anything you changed.
- Update `CHANGELOG.md` under `[Unreleased]`.
- New dependencies require prior discussion. "It would be convenient" is not a good reason.
- No `unsafe` unless there is genuinely no alternative and the reasoning is documented next to the code.
- No new public API without doc comments.
- Be respectful. Be direct. Be constructive.

---

## Ordering

Everything in this codebase that can be alphabetically ordered, is. Enum variants. Function parameters. Struct fields. Imports. Table rows. Everything.

This is not a stylistic preference. It is a rule. When things are ordered, you stop thinking about where they are and start thinking about what they do. You search for `Html` in the `ContentType` enum and your eye goes straight to the `H`s. You add a new variant and you know exactly where it lives. No debates, no "should this go before or after that" — alphabetical order is always the right answer and it is never wrong.

If you open a PR and something that could be alphabetically ordered is not, it will be sent back.

---

## People

### Maintainer

**benjamin_pla** — [benjaminpla.dev@gmail.com](mailto:benjaminpla.dev@gmail.com) · [GitHub](https://github.com/benjaminPla)

### Contributors

<!-- - username · email · github · website -->
