use qhull::Qh;
use rand::{rngs::StdRng, Rng, SeedableRng};
use svg::node::element::{Circle, Rectangle};

fn main() {
    let points = points();

    let convex_hull = Qh::builder()
        .build_from_iter(points.iter().cloned())
        .unwrap();

    let segments = convex_hull.simplices().count();
    eprintln!("Convex hull has {} segments", segments);

    let triangulation = Qh::new_delaunay(points.iter().cloned()).unwrap();

    let triangles = triangulation.simplices().filter(|s| !s.upper_delaunay()).count();
    eprintln!("Triangulation has {} triangles", triangles);

    let mut doc = svg::Document::new()
        .set("width", "400")
        .set("height", "400")
        .set("viewBox", (0, 0, 100, 100));

    doc = doc.add(Rectangle::new()
        .set("x", 0)
        .set("y", 0)
        .set("width", 100)
        .set("height", 100)
        .set("fill", "white"));

    for point in &points {
        doc = doc.add(Circle::new()
            .set("cx", point[0])
            .set("cy", point[1])
            .set("r", 0.5)
            .set("fill", "black"));
    }

    // draw the triangulation
    for s in triangulation.simplices().filter(|s| !s.upper_delaunay()) {
        let vertices = s
            .vertices().unwrap().iter()
            .map(|v| v.point().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(vertices.len(), 3);
        for i in 0..3 {
            doc = doc.add(svg::node::element::Line::new()
                .set("x1", vertices[i][0])
                .set("y1", vertices[i][1])
                .set("x2", vertices[(i + 1) % 3][0])
                .set("y2", vertices[(i + 1) % 3][1])
                .set("stroke", "blue")
                .set("stroke-width", 0.25));
        }
    }

    // draw the convex hull
    for s in convex_hull.simplices() {
        let vertices = s
            .vertices().unwrap().iter()
            .map(|v| v.point().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(vertices.len(), 2);
        let a = vertices[0];
        let b = vertices[1];
        doc = doc.add(svg::node::element::Line::new()
            .set("x1", a[0])
            .set("y1", a[1])
            .set("x2", b[0])
            .set("y2", b[1])
            .set("stroke", "red")
            .set("stroke-width", 0.5));
    }

    svg::write(std::io::stdout(), &doc).unwrap()
}

fn points() -> Vec<[f64; 2]> {
    let mut rng = StdRng::seed_from_u64(42);

    (0..20)
        .map(move |_| [
            rng.gen_range(5.0..95.0),
            rng.gen_range(5.0..95.0),
        ])
        .collect()
}