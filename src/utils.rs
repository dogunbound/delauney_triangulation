use sfml::{
    graphics::{
        CircleShape, Color, PrimitiveType, RenderTarget, RenderWindow, Shape, Transformable, Vertex,
    },
    system::Vector2f,
};

pub fn display_vertices_as_small_yellow_circles(window: &mut RenderWindow, vertices: &[Vector2f]) {
    for point in vertices {
        let mut circle = CircleShape::new(2., 20);
        circle.set_origin(Vector2f::new(1., 1.));
        circle.set_position(*point);
        circle.set_fill_color(Color::YELLOW);

        window.draw_circle_shape(&circle, &Default::default());
    }
}

pub fn display_triangles(window: &mut RenderWindow, triangles: &[[Vector2f; 3]], color: Color) {
    let mut vertex_array = vec![];
    for triangle in triangles {
        vertex_array.push(Vertex::with_pos_color(triangle[0], color));
        vertex_array.push(Vertex::with_pos_color(triangle[1], color));
        vertex_array.push(Vertex::with_pos_color(triangle[1], color));
        vertex_array.push(Vertex::with_pos_color(triangle[2], color));
        vertex_array.push(Vertex::with_pos_color(triangle[2], color));
        vertex_array.push(Vertex::with_pos_color(triangle[0], color));
    }

    let vertex_array: Vec<Vertex> = triangles
        .into_iter()
        .flat_map(|triangle| {
            [
                Vertex::with_pos_color(triangle[0], color),
                Vertex::with_pos_color(triangle[1], color),
                Vertex::with_pos_color(triangle[1], color),
                Vertex::with_pos_color(triangle[2], color),
                Vertex::with_pos_color(triangle[2], color),
                Vertex::with_pos_color(triangle[0], color),
            ]
        })
        .collect();

    window.draw_primitives(&vertex_array, PrimitiveType::LINES, &Default::default());
}
