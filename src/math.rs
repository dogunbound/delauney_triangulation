use sfml::system::Vector2;

#[allow(non_snake_case)]
#[must_use]
pub fn cosine_rule_solved_for_angle_a(a: f64, b: f64, c: f64) -> f64 {
    // a^2 = b^2 + c^2 - 2ab * cos(A) is the law of cosines
    // End formula is: A = arccos((b^2 + c^2 - a^2) / (2bc))

    let numerator = b * b + c * c - a * a;
    let denominator = 2. * b * c;
    let division_result = numerator / denominator;
    let A = division_result.acos();

    A
}

#[must_use]
pub fn euclidian_distance(a: Vector2<f64>, b: Vector2<f64>) -> f64 {
    let diff = a - b;
    let squared_distance = diff.x * diff.x + diff.y * diff.y;
    let euclidian_distance = squared_distance.sqrt();

    euclidian_distance
}

#[allow(non_snake_case)]
#[must_use]
pub fn calculate_angles_of_triangle(triangle: [Vector2<f64>; 3]) -> (f64, f64, f64) {
    // Vertices of triangle
    let (a, b, c) = (triangle[0], triangle[1], triangle[2]);
    // length of each side
    let (side_a, side_b, side_c) = (
        euclidian_distance(b, c),
        euclidian_distance(c, a),
        euclidian_distance(a, b),
    );

    // angle of each vertex on triangle respective to the vertices of the triangle itself
    let (A, B, C) = (
        cosine_rule_solved_for_angle_a(side_a, side_b, side_c),
        cosine_rule_solved_for_angle_a(side_b, side_c, side_a),
        cosine_rule_solved_for_angle_a(side_c, side_a, side_b),
    );

    (A, B, C)
}

#[allow(non_snake_case)]
#[must_use]
pub fn circumcenter_of_triangle(triangle: [Vector2<f64>; 3]) -> Vector2<f64> {
    // x = (x1 * sin(2A) + x2 * sin(2B) + x3 * sin(2C)) / (sin(2A) + sin(2B) + sin(2C))
    // y = (y1 * sin(2A) + y2 * sin(2B) + y3 * sin(2C)) / (sin(2A) + sin(2B) + sin(2C))
    let (a, b, c) = (triangle[0], triangle[1], triangle[2]);
    let (A, B, C) = calculate_angles_of_triangle(triangle);

    // sin(2*angle) of each angle
    let (dA, dB, dC) = ((2. * A).sin(), (2. * B).sin(), (2. * C).sin());

    let denominator = dA + dB + dC;
    let numerator = Vector2::new(
        a.x * dA + b.x * dB + c.x * dC,
        a.y * dA + b.y * dB + c.y * dC,
    );

    let circumcenter = numerator / denominator;

    circumcenter
}

#[must_use]
pub fn get_edges_from_triangle(triangle: [Vector2<f64>; 3]) -> [(Vector2<f64>, Vector2<f64>); 3] {
    [
        (triangle[0], triangle[1]),
        (triangle[1], triangle[2]),
        (triangle[2], triangle[0]),
    ]
}

#[must_use]
pub fn edges_are_equal(
    edge1: (Vector2<f64>, Vector2<f64>),
    edge2: (Vector2<f64>, Vector2<f64>),
) -> bool {
    let flipped_edge1 = (edge1.1, edge1.0);
    edge1 == edge2 || flipped_edge1 == edge2
}
