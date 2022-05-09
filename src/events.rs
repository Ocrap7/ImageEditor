use nannou::{lyon::geom::{euclid::{Point2D, Vector2D, UnknownUnit}, Vector}, state::mouse::ButtonMap};

use crate::ui::View;


pub struct Mouse {
    pub point: Point2D<i32, UnknownUnit>,
    pub buttons: ButtonMap
}

impl Mouse {
    pub fn new(point: Point2D<i32, UnknownUnit>, buttons: ButtonMap) -> Mouse {
        Mouse {
            point,
            buttons
        }
    }

    pub fn translate(&self, view: &impl View) -> Vector2D<i32, UnknownUnit> {
        let rect = view.get_rect();
        -(rect.origin - self.point)
    }
}