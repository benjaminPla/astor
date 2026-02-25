# Contributing to tsu

tsu exists for one reason: Rust services behind a reverse proxy. Not Rust services that *are* a reverse proxy. Keep that in mind before opening a PR.

If your idea belongs in nginx config, it does not belong here.
If your idea makes tsu better at being tsu, welcome.

---

## Table of contents

- [Code of conduct](#code-of-conduct)
- [Coding conventions](#coding-conventions)
- [Development setup](#development-setup)
- [Getting started](#getting-started)
- [People](#people)
- [Reporting issues](#reporting-issues)
- [Submitting a pull request](#submitting-a-pull-request)
- [Suggesting features](#suggesting-features)
- [Versioning](#versioning)

---

## Code of conduct

Be respectful. Be direct. Be constructive. Harassment of any kind is not tolerated.

---

## Getting started

1. **Fork** the repository.
2. **Clone** your fork.

```sh
git clone https://github.com/<your-username>/tsu.git
cd tsu
```

3. **Create a branch** from `master`.

```sh
git checkout -b feat/my-thing
```

---

## Reporting issues

Before filing a bug:

- Search existing issues — it may already be tracked.
- Reproduce with a minimal example. "It doesn't work" is not a bug report.

A good bug report includes:

- tsu version (`Cargo.toml`)
- Rust toolchain version (`rustc --version`)
- A minimal reproducer
- What you expected vs. what actually happened

---

## Suggesting features

Open a GitHub Discussion or an issue tagged `enhancement`. Before you do, ask yourself:

- Does the reverse proxy already handle this? (If yes, the answer is no.)
- Is this useful to most tsu users, or just your specific setup?
- Can it be opt-in via a Cargo feature flag?

Small, focused changes that do one thing well. Not large multi-feature PRs that do several things loosely.

---

## Submitting a pull request

Before submitting:

```sh
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

All three must pass. No exceptions.

Also:

- Add or update tests for anything you changed.
- Update `CHANGELOG.md` under `[Unreleased]`.
- Add yourself to the [Contributors](#people) list below, alphabetically by username.
- Open the PR against `master` with a clear title.

PR title format:

```
<type>: <short description>

# types: feat, fix, refactor, docs, test, chore
```

---

## Development setup

```sh
# Run the example server
RUST_LOG=info cargo run --example basic

# Run tests
cargo test

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt
```

tsu's dependencies in production code: `tokio`, `matchit`, `tracing`. That's three. New dependencies require a very good reason and prior discussion. "It would be convenient" is not a good reason.

---

## Coding conventions

- No `unsafe` unless there is genuinely no alternative and the reasoning is documented next to the code.
- No macros for things that can be functions.
- No new public API without doc comments.
- Keep the public surface small. When in doubt, keep it private.
- `#[inline]` on trivial hot-path methods (response constructors, router lookup).
- Everything that can be alphabetically ordered is alphabetically ordered. Enums, imports, function parameters, struct fields. Yes, all of it.

---

## Versioning

tsu follows [Semantic Versioning](https://semver.org):

| Change | Bump |
|---|---|
| Breaking public API change | major (`1.0.0 → 2.0.0`) |
| New backwards-compatible feature | minor (`0.1.0 → 0.2.0`) |
| Bug fix, internal refactor, docs | patch (`0.1.0 → 0.1.1`) |

While in `0.x`, minor bumps may include breaking changes. This will be clearly noted in `CHANGELOG.md`. Every release is tagged `vX.Y.Z` and published to [crates.io](https://crates.io/crates/tsu).

---

## People

### Maintainer

**benjamin_pla** — [benjaminpla.dev@gmail.com](mailto:benjaminpla.dev@gmail.com) · [GitHub](https://github.com/benjaminPla)

### Contributors

If your PR is merged, add yourself here. Alphabetical by username. One line.

<!-- - your_username — what you did -->
