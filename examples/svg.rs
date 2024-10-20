use qhull::Qh;
use rand::{rngs::StdRng, Rng, SeedableRng};
use svg::node::element::{Circle, Rectangle};

fn main() {
    let convex_hull = Qh::builder()
        .build_from_iter(points())
        .unwrap();

    let simplices = convex_hull.simplices().count();
    eprintln!("Convex hull has {simplices} segments and {} vertices", convex_hull.num_vertices());

    let triangulation = Qh::new_delaunay(points()).unwrap();

    let triangles = triangulation.simplices().filter(|s| !s.upper_delaunay()).count();
    eprintln!("Triangulation has {triangles} triangles and {} vertices", triangulation.num_vertices());

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

    for (i, point) in points().enumerate() {
        doc = doc.add(Circle::new()
            .set("cx", point[0])
            .set("cy", point[1])
            .set("r", 1.0)
            .set("fill", "black"));
        doc = doc.add(svg::node::element::Text::new(format!("{i}").as_str())
            .set("x", point[0] + 2.0)
            .set("y", point[1])
            .set("font-size", 5)
            .set("fill", "black"));
    }

    eprintln!("drawing triangles:");
    for s in triangulation.simplices().filter(|s| !s.upper_delaunay()) {
        eprintln!(
            "- {} -",
            s
                .vertices().unwrap().iter()
                .map(|v| v.index(&triangulation).unwrap().to_string())
                .collect::<Vec<_>>()
                .join(" - "),
        );
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

    eprintln!("drawing convex hull:");
    for s in convex_hull.simplices() {
        eprintln!(
            "{}",
            s
                .vertices().unwrap().iter()
                .map(|v| v.index(&convex_hull).unwrap().to_string())
                .collect::<Vec<_>>()
                .join(" - "),
        );
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

fn points() -> impl Iterator<Item = [f64; 2]> {
    let mut rng = StdRng::seed_from_u64(42);

    (0..10)
        .map(move |_| [
            rng.gen_range(5.0..95.0),
            rng.gen_range(5.0..95.0),
        ])
}