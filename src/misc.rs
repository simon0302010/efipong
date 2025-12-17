#[derive(Clone, Copy)]
pub struct Rectangle {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

pub fn rectangles_overlapping(rec1: Rectangle, rec2: Rectangle) -> bool {
    !(rec1.y > rec2.y + rec2.height
        || rec2.y > rec1.y + rec1.height
        || rec1.x + rec1.width < rec2.x
        || rec2.x + rec2.width < rec1.x)
}
