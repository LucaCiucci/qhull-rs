use qhull::*;

fn main() {
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
}