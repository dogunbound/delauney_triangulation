use sfml::{
    graphics::{
        CircleShape, Color, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Shape,
        Transformable, Vertex,
    },
    system::Vector2,
};

use crate::{
    circle::Circle,
    math::{edges_are_equal, get_edges_from_triangle},
    utils::{self, display_triangles},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum InternalState {
    #[default]
    Initial,
    GetBadTrianglesInMesh(usize),
    PolygonalHole,
    RemoveBadTrianglesFromMesh(bool),
    AddTrianglesFromPolygonEdges,
}

impl PartialOrd for InternalState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.to_usize().partial_cmp(&other.to_usize())
    }
}

impl InternalState {
    fn to_usize(&self) -> usize {
        use InternalState::*;
        match self {
            Initial => 0,
            GetBadTrianglesInMesh(_) => 1,
            PolygonalHole => 2,
            RemoveBadTrianglesFromMesh(_) => 3,
            AddTrianglesFromPolygonEdges => 4,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct DelauneyTriangulationInformation {
    state: InternalState,
    current_point_idx: usize,
    point_list: Vec<Vector2<f64>>,
    triangulation_mesh: Vec<Vertex>,
    bad_triangles_to_plot: Vec<[Vector2<f64>; 3]>,
    good_checked_triangles_to_plot: Vec<[Vector2<f64>; 3]>,
    super_triangle: Option<[Vector2<f64>; 3]>,
    circumcircles_to_plot: Vec<Circle>,
    polygon_for_new_triangles: Vec<(Vector2<f64>, Vector2<f64>)>,
}

impl DelauneyTriangulationInformation {
    pub fn reset_delauney_mesh(&mut self) {
        self.state = InternalState::Initial;
        self.current_point_idx = 0;
        self.point_list = Default::default();
        self.triangulation_mesh = Default::default();
        self.bad_triangles_to_plot = Default::default();
        self.good_checked_triangles_to_plot = Default::default();
        self.super_triangle = Default::default();
        self.circumcircles_to_plot = Default::default();
        self.polygon_for_new_triangles = Default::default();
    }

    fn clear_crap_to_plot(&mut self) {
        self.bad_triangles_to_plot = Default::default();
        self.circumcircles_to_plot = Default::default();
        self.good_checked_triangles_to_plot = Default::default();
    }

    pub fn set_point_list(&mut self, point_list: Vec<Vector2<f64>>) {
        self.point_list = point_list;
    }

    pub fn draw(&self, window: &mut RenderWindow) {
        for circle in &self.circumcircles_to_plot {
            circle.draw(window, Color::rgba(255, 215, 0, 50), Color::TRANSPARENT);
        }
        window.draw_primitives(
            &self.triangulation_mesh,
            PrimitiveType::LINES,
            &RenderStates::DEFAULT,
        );
        display_triangles(window, &self.bad_triangles_to_plot, Color::RED);
        display_triangles(window, &self.good_checked_triangles_to_plot, Color::GREEN);

        utils::display_vertices(window, &self.point_list, Color::YELLOW);

        if let Some(current_point) = self.point_list.get(self.current_point_idx) {
            let mut circle = CircleShape::new(5., 20);
            circle.set_origin(Vector2::new(circle.radius(), circle.radius()));
            circle.set_position(current_point.as_other());
            circle.set_fill_color(Color::CYAN);

            window.draw_circle_shape(&circle, &Default::default());
        }
    }

    fn add_triangle_to_mesh(&mut self, triangle: [Vector2<f64>; 3]) {
        self.triangulation_mesh
            .push(Vertex::with_pos(triangle[0].as_other()));
        self.triangulation_mesh
            .push(Vertex::with_pos(triangle[1].as_other()));
        self.triangulation_mesh
            .push(Vertex::with_pos(triangle[1].as_other()));
        self.triangulation_mesh
            .push(Vertex::with_pos(triangle[2].as_other()));
        self.triangulation_mesh
            .push(Vertex::with_pos(triangle[2].as_other()));
        self.triangulation_mesh
            .push(Vertex::with_pos(triangle[0].as_other()));
    }

    fn iter_triangles_in_mesh(&self) -> impl Iterator<Item = [Vector2<f64>; 3]> + '_ {
        self.triangulation_mesh.chunks(6).map(|chunk| {
            [
                chunk[0].position.as_other(),
                chunk[1].position.as_other(),
                chunk[3].position.as_other(),
            ]
        })
    }

    fn remove_triangle_from_mesh(&mut self, triangle: [Vector2<f64>; 3]) {
        let mut removal_index = Default::default();
        for (idx, mesh_triangle) in self.iter_triangles_in_mesh().enumerate() {
            if mesh_triangle == triangle {
                removal_index = Some(idx * 6);
            }
        }

        if let Some(removal_index) = removal_index {
            self.triangulation_mesh
                .drain(removal_index..(removal_index + 6));
        }
    }
}

/// Delauney algorithm calculations
impl DelauneyTriangulationInformation {
    fn add_super_triangle(&mut self) {
        let mut max = Vector2::default();
        for point in &self.point_list {
            if point.x > max.x {
                max.x = point.x;
            }
            if point.y > max.y {
                max.y = point.y;
            }
        }
        max.x *= 2.;
        max.y *= 2.;

        let super_triangle = [
            Vector2::new(-1., -1.),
            Vector2::new(max.x + 3., -1.),
            Vector2::new(-1., max.y + 3.),
        ];
        self.super_triangle = Some(super_triangle);
        self.add_triangle_to_mesh(super_triangle);
    }

    fn remove_triangles_attached_to_super_triangle(&mut self) {
        let Some(super_triangle) = self.super_triangle else {
            return;
        };

        let mut triangles_to_remove = vec![];
        'triangle_loop: for triangle in self.iter_triangles_in_mesh() {
            for super_vertex in super_triangle {
                for triangle_vertex in triangle {
                    if super_vertex == triangle_vertex {
                        triangles_to_remove.push(triangle);
                        continue 'triangle_loop;
                    }
                }
            }
        }

        for triangle_to_remove in triangles_to_remove {
            self.remove_triangle_from_mesh(triangle_to_remove);
        }
    }

    fn get_all_bad_triangles_in_mesh_and_circumcircles_checked(
        &mut self,
        point: Vector2<f64>,
    ) -> Vec<[Vector2<f64>; 3]> {
        let mut bad_triangles = vec![];
        let mut crap_to_plot = None;
        let mut is_last_triangle_a_bad_triangle = false;

        for (idx, triangle) in self.iter_triangles_in_mesh().enumerate() {
            let circumcircle = Circle::from(triangle);
            is_last_triangle_a_bad_triangle = circumcircle.is_point_inside_circle(point);
            if is_last_triangle_a_bad_triangle {
                bad_triangles.push(triangle);
            }

            if let InternalState::GetBadTrianglesInMesh(current_idx) = self.state {
                if idx > current_idx {
                    crap_to_plot = Some((triangle, circumcircle));
                    break;
                }
            }
        }

        if let Some((triangle, circumcircle)) = crap_to_plot {
            self.circumcircles_to_plot.push(circumcircle);
            if is_last_triangle_a_bad_triangle {
                self.bad_triangles_to_plot.push(triangle);
            } else {
                self.good_checked_triangles_to_plot.push(triangle);
            }
            if let InternalState::GetBadTrianglesInMesh(current_idx) = &mut self.state {
                *current_idx += 1;
            }
            return bad_triangles;
        }

        if self.state < InternalState::PolygonalHole {
            self.state = InternalState::PolygonalHole;
        }

        bad_triangles
    }

    fn polygonal_hole_boundary(
        bad_triangles: &[[Vector2<f64>; 3]],
    ) -> Vec<(Vector2<f64>, Vector2<f64>)> {
        let mut polygon = vec![];
        for (idx, triangle) in bad_triangles.iter().enumerate() {
            let triangle_edges = get_edges_from_triangle(*triangle);
            let mut triangle_edge_that_has_match = (false, false, false);

            for (idx_c, triangle_c) in bad_triangles.iter().enumerate() {
                if idx == idx_c {
                    continue;
                }

                let triangle_edges_c = get_edges_from_triangle(*triangle_c);

                for edge in triangle_edges_c {
                    triangle_edge_that_has_match.0 |= edges_are_equal(triangle_edges[0], edge);
                    triangle_edge_that_has_match.1 |= edges_are_equal(triangle_edges[1], edge);
                    triangle_edge_that_has_match.2 |= edges_are_equal(triangle_edges[2], edge);
                }
            }

            if !triangle_edge_that_has_match.0 {
                polygon.push(triangle_edges[0]);
            }

            if !triangle_edge_that_has_match.1 {
                polygon.push(triangle_edges[1]);
            }

            if !triangle_edge_that_has_match.2 {
                polygon.push(triangle_edges[2]);
            }
        }

        polygon
    }

    pub fn remove_all_bad_triangles_from_mesh(&mut self, bad_triangles: &[[Vector2<f64>; 3]]) {
        if let InternalState::RemoveBadTrianglesFromMesh(ran_through_removal_once) = self.state {
            if !ran_through_removal_once {
                self.state = InternalState::RemoveBadTrianglesFromMesh(true);
            } else {
                self.state = InternalState::AddTrianglesFromPolygonEdges;
                return;
            }
        }
        for triangle in bad_triangles {
            self.remove_triangle_from_mesh(*triangle);
        }

        self.state = InternalState::RemoveBadTrianglesFromMesh(true);
    }

    pub fn add_triangles_from_polygon_edges(
        &mut self,
        polygon: &[(Vector2<f64>, Vector2<f64>)],
        point: Vector2<f64>,
    ) {
        for edge in polygon {
            let new_triangle = [point, edge.0, edge.1];
            self.add_triangle_to_mesh(new_triangle);
        }
    }

    /// Early return indicates a draw up\date is needed.
    ///
    /// Psuedocode reference:
    /// function BowyerWatson (pointList)
    ///     // pointList is a set of coordinates defining the points to be triangulated
    ///     triangulation := empty triangle mesh data structure
    ///     add super-triangle to triangulation // must be large enough to completely contain all the points in pointList
    ///     for each point in pointList do // add all the points one at a time to the triangulation
    ///         badTriangles := empty set
    ///         for each triangle in triangulation do // first find all the triangles that are no longer valid due to the insertion
    ///             if point is inside circumcircle of triangle
    ///                 add triangle to badTriangles
    ///         polygon := empty set
    ///         for each triangle in badTriangles do // find the boundary of the polygonal hole
    ///             for each edge in triangle do
    ///                 if edge is not shared by any other triangles in badTriangles
    ///                     add edge to polygon
    ///         for each triangle in badTriangles do // remove them from the data structure
    ///             remove triangle from triangulation
    ///         for each edge in polygon do // re-triangulate the polygonal hole
    ///             newTri := form a triangle from edge to point
    ///             add newTri to triangulation
    ///     for each triangle in triangulation // done inserting points, now clean up
    ///         if triangle contains a vertex from original super-triangle
    ///             remove triangle from triangulation
    ///     return triangulation    
    pub fn update_triangulation(&mut self) {
        self.clear_crap_to_plot();

        if self.state == InternalState::Initial {
            self.add_super_triangle();
            self.state = InternalState::GetBadTrianglesInMesh(0);
            return;
        }

        let Some(point) = self.point_list.get(self.current_point_idx) else {
            self.remove_triangles_attached_to_super_triangle();
            self.bad_triangles_to_plot = Default::default();
            self.circumcircles_to_plot = Default::default();
            return;
        };
        let point = *point; // added this line to deref `point` and make it no longer linked to point list
        let bad_triangles = self.get_all_bad_triangles_in_mesh_and_circumcircles_checked(point);
        if self.state < InternalState::PolygonalHole {
            return;
        }
        let polygon = Self::polygonal_hole_boundary(&bad_triangles);
        self.remove_all_bad_triangles_from_mesh(&bad_triangles);
        if self.state <= InternalState::RemoveBadTrianglesFromMesh(true) {
            self.polygon_for_new_triangles = polygon;
            return;
        }
        let polygon = self.polygon_for_new_triangles.clone();
        self.add_triangles_from_polygon_edges(&polygon, point);
        self.current_point_idx += 1;

        self.state = InternalState::GetBadTrianglesInMesh(0);
    }
}
