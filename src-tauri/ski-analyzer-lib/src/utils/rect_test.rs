use super::rect::union_rects;
use geo::{coord, Rect};

fn rect(x1: f64, y1: f64, x2: f64, y2: f64) -> Rect {
    Rect::new(coord! { x: x1, y: y1 }, coord! { x: x2, y: y2 })
}

#[test]
fn a_contains_b() {
    let a = rect(1.0, 6.0, 10.0, 20.0);
    let b = rect(2.0, 11.0, 8.0, 19.0);
    assert_eq!(union_rects(a, b), a);
}

#[test]
fn b_contains_a() {
    let a = rect(0.0, -1.0, 10.0, 5.0);
    let b = rect(-1.0, -5.0, 15.0, 8.0);
    assert_eq!(union_rects(a, b), b);
}

#[test]
fn b_right_down_of_a() {
    let a = rect(1.0, 2.0, 10.0, 12.0);
    let b = rect(5.0, 4.0, 15.0, 13.0);
    assert_eq!(union_rects(a, b), rect(1.0, 2.0, 15.0, 13.0));
}

#[test]
fn b_left_down_of_a() {
    let a = rect(0.0, -1.0, 2.0, 3.0);
    let b = rect(-5.0, 0.0, 1.0, 5.0);
    assert_eq!(union_rects(a, b), rect(-5.0, -1.0, 2.0, 5.0));
}

#[test]
fn b_left_up_of_a() {
    let a = rect(1.0, 10.0, 12.0, 13.0);
    let b = rect(0.0, 2.0, 5.0, 11.0);
    assert_eq!(union_rects(a, b), rect(0.0, 2.0, 12.0, 13.0));
}

#[test]
fn b_right_up_of_a() {
    let a = rect(3.0, 1.0, 20.0, 10.0);
    let b = rect(9.0, -10.0, 15.0, 2.0);
    assert_eq!(union_rects(a, b), rect(3.0, -10.0, 20.0, 10.0));
}
