# qhull-rs

Safe Rust bindings for [Qhull](http://www.qhull.org/), a library for convex hulls,
Delaunay triangulations, Voronoi diagrams, and halfspace intersections in two or
more dimensions.

[![Crates.io Version](https://img.shields.io/crates/v/qhull)](https://crates.io/crates/qhull)
[![Build Status](https://img.shields.io/github/actions/workflow/status/LucaCiucci/qhull-rs/rust.yml)](https://github.com/LucaCiucci/qhull-rs/actions)
[![docs.rs](https://img.shields.io/docsrs/qhull)](https://docs.rs/qhull)

Qhull implements the Quickhull algorithm and handles floating-point roundoff
errors. See the [Qhull documentation](http://www.qhull.org/) for its supported
operations and limitations.

## Quick start

```sh
cargo run --example hull
```

### Command-line tools

The crate also provides command-line tools from the original Qhull source:

- `qconvex`
- `qdelaunay`
- `qhalf`
- `qhull`
- `qvoronoi`
- `rbox`

Install them with:

```sh
cargo install qhull
```

## Usage

Add this to your `Cargo.toml`:

```toml
qhull = "0.4"
```

For the current development version:

```toml
[dependencies]
qhull = { git = "https://github.com/LucaCiucci/qhull-rs" }
```

Read the [API documentation](https://docs.rs/qhull) for the full interface.

### Example

A 2D convex hull:
```rust
use qhull::Qh;

let qh = Qh::builder()
    .compute(true)
    .build_from_iter([
        [0.0, 0.0],
        [1.0, 0.0],
        [0.0, 1.0],
        [0.25, 0.25],
    ]).unwrap();

for simplex in qh.simplices() {
    let vertices = simplex
        .vertices().unwrap()
        .iter()
        .map(|v| v.index(&qh).unwrap())
        .collect::<Vec<_>>();

    println!("{:?}", vertices);
}
```

See the [`examples`] module/folder for more examples.

## Development

Building requires a Rust toolchain, a C compiler, and `libclang`.

Clone the repository with its Qhull submodule:
```sh
git clone --recurse-submodules https://github.com/LucaCiucci/qhull-rs.git
cd qhull-rs
cargo build
```

If you also install [just](https://just.systems/), you can run the CI suite locally with:
```sh
just ci
```

### Releasing

After committing and tagging the release as `v<version>`, run:

```sh
just publish
```

## License

This crate uses Qhull, please refer to the [Qhull license](http://www.qhull.org/COPYING.txt) for more information when using this crate.
