use sfml::{
    graphics::{
        CircleShape, Color, PrimitiveType, RenderTarget, RenderWindow, Shape, Transformable, Vertex,
    },
    system::{Vector2, Vector2f},
};

pub fn display_vertices(window: &mut RenderWindow, vertices: &[Vector2<f64>], color: Color) {
    for point in vertices {
        let mut circle = CircleShape::new(2., 20);
        circle.set_origin(Vector2f::new(circle.radius(), circle.radius()));
        circle.set_position(point.as_other());
        circle.set_fill_color(color);

        window.draw_circle_shape(&circle, &Default::default());
    }
}

pub fn display_triangles(window: &mut RenderWindow, triangles: &[[Vector2<f64>; 3]], color: Color) {
    let vertex_array: Vec<Vertex> = triangles
        .into_iter()
        .flat_map(|triangle| {
            [
                Vertex::with_pos_color(triangle[0].as_other(), color),
                Vertex::with_pos_color(triangle[1].as_other(), color),
                Vertex::with_pos_color(triangle[1].as_other(), color),
                Vertex::with_pos_color(triangle[2].as_other(), color),
                Vertex::with_pos_color(triangle[2].as_other(), color),
                Vertex::with_pos_color(triangle[0].as_other(), color),
            ]
        })
        .collect();

    window.draw_primitives(&vertex_array, PrimitiveType::LINES, &Default::default());
}
