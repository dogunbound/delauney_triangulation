use sfml::{
    graphics::{
        CircleShape, Color, RenderStates, RenderTarget, RenderWindow, Shape, Transformable,
    },
    system::Vector2f,
};

use crate::math::{circumcenter_of_triangle, euclidian_distance};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Circle {
    center: Vector2f,
    radius: f32,
}

impl Circle {
    pub fn is_point_inside_circle(&self, point: Vector2f) -> bool {
        let euclidian_distance = euclidian_distance(self.center, point);
        euclidian_distance < self.radius
    }

    pub fn draw(&self, window: &mut RenderWindow, fill_color: Color, outline_color: Color) {
        let mut circle = CircleShape::new(self.radius, self.radius as usize);
        circle.set_outline_thickness(1.);
        circle.set_position(self.center);
        circle.set_origin(Vector2f::new(self.radius, self.radius));
        circle.set_fill_color(fill_color);
        circle.set_outline_color(outline_color);
        window.draw_circle_shape(&circle, &RenderStates::default());
    }
}

impl From<[Vector2f; 3]> for Circle {
    fn from(triangle: [Vector2f; 3]) -> Self {
        let (side_a, side_b, side_c) = (
            euclidian_distance(triangle[0], triangle[1]),
            euclidian_distance(triangle[1], triangle[2]),
            euclidian_distance(triangle[2], triangle[0]),
        );

        // r = (abc)/((a+b+c)(b+c-a)(c+a-b)(a+b-c))^.5
        let numerator = side_a * side_b * side_c;
        let denominator1 = side_a + side_b + side_c;
        let denominator2 = side_b + side_c - side_a;
        let denominator3 = side_c + side_a - side_b;
        let denominator4 = side_a + side_b - side_c;
        let denominator = (denominator1 * denominator2 * denominator3 * denominator4).sqrt();
        let radius = numerator / denominator;

        let center = circumcenter_of_triangle(triangle);

        Circle { center, radius }
    }
}
