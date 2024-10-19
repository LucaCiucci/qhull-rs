use qhull::Qh;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let qh = Qh::new_delaunay([[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [0.25, 0.25]])?;

    for simplex in qh.simplices().filter(|f| !f.upper_delaunay()) {
        println!(
            "{:?}",
            simplex
                .vertices()
                .unwrap()
                .iter()
                .map(|v| v.id())
                .collect::<Vec<_>>()
        );
    }

    let mut simplices = qh
        .simplices()
        .filter(|f| !f.is_sentinel() && !f.upper_delaunay())
        .map(|f| {
            f.vertices()
                .unwrap()
                .iter()
                .map(|v| v.id() - 1)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    simplices.iter_mut().for_each(|s| s.sort());
    simplices.sort();
    assert_eq!(
        simplices,
        vec![vec![0, 1, 3], vec![0, 2, 3], vec![1, 2, 3],]
    );

    Ok(())
}
