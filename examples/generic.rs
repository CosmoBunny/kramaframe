use std::{
    fmt::Display,
    ops::{Add, Mul, Sub},
};

use kramaframe::{BTframelist, KramaFrame, keylist::TRES16Bits};

#[derive(Clone, Copy)]
struct Point {
    x: f32,
    y: f32,
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for Point {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Point {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Point {
    fn new(x: f32, y: f32) -> Self {
        Point { x, y }
    }
}

fn main() {
    let mut animation_instance: KramaFrame<_, BTframelist<_, i16>> = KramaFrame::default();
    animation_instance
        .classlist
        .insert("point", kramaframe::prelude::KeyFrameFunction::EaseIn);
    animation_instance.insert_new_id("point", 1, TRES16Bits::from_millis(600));
    animation_instance.restart_progress("point", 1);

    for i in 0..=90 {
        animation_instance.update_progress(TRES16Bits::from_millis(16));
        let point_value = animation_instance.get_generic_value_by_rangeinclusive(
            "point",
            1,
            Point::new(3.0, 4.0)..=Point::new(5.0, 6.0),
        );
        println!("Frame {} point position is {}", i, point_value)
    }
}
