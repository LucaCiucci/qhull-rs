use qhull::Qh;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let qh = Qh::builder().compute(true).build_from_iter([
        [0.0, 0.0],
        [1.0, 0.0],
        [0.0, 1.0],
        [0.25, 0.25],
    ])?;

    assert_eq!(qh.num_facets(), 3);

    for simplex in qh.simplices() {
        let vertices = simplex
            .vertices()
            .unwrap()
            .iter()
            .map(|v| v.index(&qh).unwrap())
            .collect::<Vec<_>>();

        println!("{:?}", vertices);
    }

    Ok(())
}
