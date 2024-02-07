# qhull-rs
 Rust [Qhull](http://www.qhull.org/) bindings

This is a safe wrapper around the `qhull-sys` crate, it is not feature complete yet, you might prefer to use the raw `qhull-sys` crate if you need more control.

> ⚠️**Warning**⚠️  
> Errors are not handled properly yet.

<!--
## Usage

Add this to your `Cargo.toml`:

```toml
qhull = "0.1"
```

For the current development version:
```toml
[dependencies]
qhull = { git = "https://github.com/LucaCiucci/qhull-rs" }
```
-->

## Examples

This creates a convex hull of a set of points in 2D:
```rust
use qhull::*;

let qh = Qh::builder(2)
    .build_from_iter([
        [0.0, 0.0],
        [1.0, 0.0],
        [0.0, 1.0],
        [0.25, 0.25],
    ]);

assert_eq!(qh.num_faces(), 3);

for simplex in qh.simplices() {
    println!("{:?}", simplex.vertices().map(|v| v.id()).collect::<Vec<_>>());
}
```

To create a delaunay triangulation, you should use the [`Qh::new_delaunay`] method.

See the `examples` directory for more examples, you can run them with `cargo run --example <example_name>`, for example:

```sh
cargo run --example delaunay
```

## Error handling

Qhull uses `setjmp`/`longjmp` for error handling, this is not currently supported in Rust, so errors are not handled properly yet.

Relevant links:
- https://github.com/rust-lang/rfcs/issues/2625
- https://docs.rs/setjmp/0.1.4/setjmp/index.html

To walk around [\#2625](https://github.com/rust-lang/rfcs/issues/2625) we might use some custom C code, but this would require some work I'm not willing to do right now.

## License

This crate uses Qhull, please refer to the [Qhull license](http://www.qhull.org/COPYING.txt) for more information when using this crate.