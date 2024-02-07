use qhull::*;

fn main() {
    let qh = Qh::new_delaunay([
        [0.0, 0.0],
        [1.0, 0.0],
        [0.0, 1.0],
        [0.25, 0.25],
    ]);

    for simplex in qh.simplices() {
        println!("{:?}", simplex.vertices().map(|v| v.id()).collect::<Vec<_>>());
    }

    let mut simpleces = qh
        .simplices()
        .map(|f| f.vertices().map(|v| v.id() - 1).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    simpleces.iter_mut().for_each(|s| s.sort());
    simpleces.sort();
    assert_eq!(simpleces, vec![
        vec![0, 1, 2],
        vec![0, 1, 3],
        vec![0, 2, 3],
        vec![1, 2, 3],
    ]);
}