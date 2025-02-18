# qhull-rs
Safe Rust [Qhull](http://www.qhull.org/) bindings

[![Crates.io Version](https://img.shields.io/crates/v/qhull)](https://crates.io/crates/qhull)
[![Build Status](https://img.shields.io/github/actions/workflow/status/LucaCiucci/qhull-rs/rust.yml)](https://github.com/LucaCiucci/qhull-rs/actions)
[![docs.rs](https://img.shields.io/docsrs/qhull)](https://docs.rs/qhull)



> [Qhull](http://www.qhull.org/) computes the **convex hull**, **Delaunay** triangulation, **Voronoi** diagram, **halfspace intersection** about a point, **furthest-site Delaunay** triangulation, and furthest-site Voronoi diagram. The source code runs in **2-d**, **3-d**, **4-d**, and **higher dimensions**. Qhull implements the **Quickhull algorithm** for computing the convex hull. It handles roundoff errors from floating point arithmetic. It computes volumes, surface areas, and approximations to the convex hull.
> 
> Qhull does not support triangulation of non-convex surfaces, mesh generation of non-convex objects, medium-sized inputs in 9-D and higher, alpha shapes, weighted Voronoi diagrams, Voronoi volumes, or constrained Delaunay triangulations.
>
> &nbsp;&nbsp;&nbsp;&nbsp;\- [_Qhull main page_](http://www.qhull.org/) (retrieved<!--accessed?--> 2024-09-02)

## Quick start

```sh
cargo run --example hull
```

### Binaries

`qhull-rs` provides some binary targets from the original Qhull source code:
- `qconvex`
- `qdelaunay`
- `qhalf`
- `qhull`
- `qvoronoi`
- `rbox`

To get them:
```sh
cargo install qhull
qhull
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

## License

This crate uses Qhull, please refer to the [Qhull license](http://www.qhull.org/COPYING.txt) for more information when using this crate.