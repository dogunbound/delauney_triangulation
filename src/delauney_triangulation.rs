use sfml::{
    graphics::{Color, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Vertex},
    system::Vector2f,
};

use crate::{
    circle::Circle,
    math::{edges_are_equal, get_edges_from_triangle},
    utils::{self, display_triangles},
};

#[derive(Default, Debug, Clone)]
pub struct DelauneyTriangulationInformation {
    current_point_idx: usize,
    point_list: Vec<Vector2f>,
    triangulation_mesh: Vec<Vertex>,
    bad_triangles_to_plot: Vec<[Vector2f; 3]>,
    super_triangle: Option<[Vector2f; 3]>,
    circumcircles_to_plot: Vec<Circle>,
}

impl DelauneyTriangulationInformation {
    pub fn reset_delauney_mesh(&mut self) {
        self.point_list = Default::default();
        self.triangulation_mesh = Default::default();
        self.bad_triangles_to_plot = Default::default();
        self.circumcircles_to_plot = Default::default();
        self.current_point_idx = 0;
        self.super_triangle = Default::default();
    }

    pub fn set_point_list(&mut self, point_list: Vec<Vector2f>) {
        self.point_list = point_list;
    }

    pub fn draw(&self, window: &mut RenderWindow) {
        for circle in &self.circumcircles_to_plot {
            // circle.draw(window, Color::rgba(255, 215, 0, 50), Color::TRANSPARENT);
        }
        window.draw_primitives(
            &self.triangulation_mesh,
            PrimitiveType::LINES,
            &RenderStates::DEFAULT,
        );
        display_triangles(window, &self.bad_triangles_to_plot, Color::RED);

        utils::display_vertices_as_small_yellow_circles(window, &self.point_list);
    }

    fn add_triangle_to_mesh(&mut self, triangle: [Vector2f; 3]) {
        self.triangulation_mesh.push(Vertex::with_pos(triangle[0]));
        self.triangulation_mesh.push(Vertex::with_pos(triangle[1]));
        self.triangulation_mesh.push(Vertex::with_pos(triangle[1]));
        self.triangulation_mesh.push(Vertex::with_pos(triangle[2]));
        self.triangulation_mesh.push(Vertex::with_pos(triangle[2]));
        self.triangulation_mesh.push(Vertex::with_pos(triangle[0]));
    }

    fn iter_triangles_in_mesh(&self) -> impl Iterator<Item = [Vector2f; 3]> + '_ {
        self.triangulation_mesh
            .chunks(6)
            .map(|chunk| [chunk[0].position, chunk[1].position, chunk[3].position])
    }

    fn remove_triangle_from_mesh(&mut self, triangle: [Vector2f; 3]) {
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
        let mut max = Vector2f::default();
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
            Vector2f::new(-1., -1.),
            Vector2f::new(max.x + 3., -1.),
            Vector2f::new(-1., max.y + 3.),
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
        &self,
        point: Vector2f,
    ) -> (Vec<[Vector2f; 3]>, Vec<Circle>) {
        let mut bad_triangles = vec![];
        let mut circumcircles = vec![];

        for triangle in self.iter_triangles_in_mesh() {
            let circumcircle = Circle::from(triangle);

            if circumcircle.is_point_inside_circle(point) {
                circumcircles.push(circumcircle);
                bad_triangles.push(triangle);
            }
        }

        (bad_triangles, circumcircles)
    }

    fn polygonal_hole_boundary(bad_triangles: &[[Vector2f; 3]]) -> Vec<(Vector2f, Vector2f)> {
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

    pub fn remove_all_bad_triangles_from_mesh(&mut self, bad_triangles: &[[Vector2f; 3]]) {
        for triangle in bad_triangles {
            self.remove_triangle_from_mesh(*triangle);
        }
    }

    pub fn add_triangles_from_polygon_edges(
        &mut self,
        polygon: &[(Vector2f, Vector2f)],
        point: Vector2f,
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
        if self.triangulation_mesh.is_empty() {
            self.add_super_triangle();
            return;
        }

        let Some(point) = self.point_list.get(self.current_point_idx) else {
            self.remove_triangles_attached_to_super_triangle();
            self.bad_triangles_to_plot = Default::default();
            self.circumcircles_to_plot = Default::default();
            return;
        };
        let point = *point; // added this line to deref `point` and make it no longer linked to point list
        self.current_point_idx += 1;
        let (bad_triangles, circumcircles) =
            self.get_all_bad_triangles_in_mesh_and_circumcircles_checked(point);
        let polygon = Self::polygonal_hole_boundary(&bad_triangles);
        self.remove_all_bad_triangles_from_mesh(&bad_triangles);
        self.add_triangles_from_polygon_edges(&polygon, point);

        self.bad_triangles_to_plot = bad_triangles;
        self.circumcircles_to_plot = circumcircles;
    }
}
