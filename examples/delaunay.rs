use std::error::Error;
use qhull::Qh;

fn main() -> Result<(), Box<dyn Error>> {
    let qh = Qh::new_delaunay([
        [0.0, 0.0],
        [1.0, 0.0],
        [0.0, 1.0],
        [0.25, 0.25],
    ])?;

    for simplex in qh.simplices() {
        println!("{:?}", simplex.vertices().iter().map(|v| v.id()).collect::<Vec<_>>());
    }

    let mut simplices = qh
        .simplices()
        .map(|f| f.vertices().iter().map(|v| v.id() - 1).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    simplices.iter_mut().for_each(|s| s.sort());
    simplices.sort();
    assert_eq!(simplices, vec![
        vec![0, 1, 2],
        vec![0, 1, 3],
        vec![0, 2, 3],
        vec![1, 2, 3],
    ]);

    Ok(())
}