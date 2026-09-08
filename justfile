default: ci

# Continuous Integration tasks: use this to run all the checks that the CI pipeline would run
ci: ci-check ci-test ci-formatted ci-clippy ci-docs

ci-check:
    cargo check --verbose
    cargo build --verbose
    cargo build --verbose --release

ci-test:
    cargo test --verbose --all
    cargo test --verbose --all --release

ci-formatted:
    cargo fmt -- --check

ci-clippy:
    cargo clippy --all-targets --all-features -- -D warnings

ci-docs:
    cargo doc --verbose --all-features

# Verify that the current commit is ready to be published as a tagged release.
release-check:
    ./scripts/release-check.sh
    just ci
    cargo publish --package qhull-sys --locked --dry-run

# Publish qhull-sys before qhull so the latter can resolve its exact dependency.
publish: release-check
    cargo publish --package qhull-sys --locked
    cargo publish --package qhull --locked
