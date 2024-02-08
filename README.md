# qhull-rs
 Rust [Qhull](http://www.qhull.org/) bindings

This is a safe wrapper around the `qhull-sys` crate, it is not feature complete yet, you might prefer to use the raw `qhull-sys` crate if you need more control.

## Usage

Add this to your `Cargo.toml`:

```toml
qhull = "0.2"
```

For the current development version:
```toml
[dependencies]
qhull = { git = "https://github.com/LucaCiucci/qhull-rs" }
```

### Examples

This creates a convex hull of a set of points in 2D:
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
        .vertices()
        .map(|v| v.id())
        .collect::<Vec<_>>();

    println!("{:?}", vertices);
}
```

To create a delaunay triangulation, you should use the [`Qh::new_delaunay`] method.

See the `examples` directory for more examples, you can run them with `cargo run --example <example_name>`, for example:

```sh
cargo run --example delaunay
```

## License

This crate uses Qhull, please refer to the [Qhull license](http://www.qhull.org/COPYING.txt) for more information when using this crate.