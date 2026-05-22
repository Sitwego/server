use geo::{Distance, Haversine, Point};
use kdtree::KdTree;
use kdtree::distance::squared_euclidean;
use redis_store::r_types::{GeoPoint, Latitude, Longitude};
use std::f64;

#[derive(Debug, Clone)]
pub struct Polyline {
    pub points: Vec<GeoPoint>,
    pub kdtree: KdTree<f64, usize, [f64; 2]>,
}

impl Polyline {
    pub fn new(points: Vec<GeoPoint>) -> Self {
        // Create a k-d tree for the points (O(n log n))
        let mut kdtree = KdTree::new(2);
        for (i, p) in points.iter().enumerate() {
            kdtree.add([p.lat.0, p.lon.0], i).unwrap();
        }
        Polyline { points, kdtree }
    }

    pub fn find_closest_point(&self, current_location: GeoPoint) -> GeoPoint {
        // Find the nearest point in the k-d tree (O(log n))
        let nearest = self
            .kdtree
            .nearest(
                &[current_location.lat.0, current_location.lon.0],
                2,
                &squared_euclidean,
            )
            .unwrap();

        let mut min_distance_sq = f64::MAX;
        let mut closest_point = self.points[*nearest[0].1]; // nearest vertex

        for &(_, idx) in nearest.iter() {
            if *idx > 0 {
                let p1 = &self.points[idx - 1];
                let p2 = &self.points[*idx];
                let closest = self.find_closest_point_on_segment(
                    p1,
                    p2,
                    &current_location,
                );
                let dist_sq = squared_euclidean(
                    &[closest.lat.0, closest.lon.0],
                    &[current_location.lat.0, current_location.lon.0],
                );
                if dist_sq < min_distance_sq {
                    min_distance_sq = dist_sq;
                    closest_point = closest;
                }
            }
            if *idx < self.points.len() - 1 {
                let p1 = &self.points[*idx];
                let p2 = &self.points[idx + 1];
                let closest = self.find_closest_point_on_segment(
                    p1,
                    p2,
                    &current_location,
                );
                let dist_sq = squared_euclidean(
                    &[closest.lat.0, closest.lon.0],
                    &[current_location.lat.0, current_location.lon.0],
                );
                if dist_sq < min_distance_sq {
                    min_distance_sq = dist_sq;
                    closest_point = closest;
                }
            }
        }

        closest_point
    }

    fn find_closest_point_on_segment(
        &self,
        p1: &GeoPoint,
        p2: &GeoPoint,
        p: &GeoPoint,
    ) -> GeoPoint {
        let x = p.lon.0;
        let y = p.lat.0;
        let x1 = p1.lon.0;
        let y1 = p1.lat.0;
        let x2 = p2.lon.0;
        let y2 = p2.lat.0;

        let a = x - x1;
        let b = y - y1;
        let c = x2 - x1;
        let d = y2 - y1;

        let dot = a * c + b * d;
        let len_sq = c * c + d * d;
        let mut param = -1.0;

        if len_sq != 0.0 {
            param = dot / len_sq;
        }

        let xx;
        let yy;

        if param < 0.0 {
            xx = x1;
            yy = y1;
        } else if param > 1.0 {
            xx = x2;
            yy = y2;
        } else {
            xx = x1 + param * c;
            yy = y1 + param * d;
        }

        GeoPoint {
            lat: Latitude(yy),
            lon: Longitude(xx),
        }
    }

    pub fn get_distance_between_two_points_in_meters(
        &self,
        a: GeoPoint,
        b: GeoPoint,
    ) -> f64 {
        let point_a = Point::new(a.lat.0, a.lon.0);
        let point_b = Point::new(b.lat.0, b.lon.0);
        Haversine::distance(point_a, point_b)
    }
}
