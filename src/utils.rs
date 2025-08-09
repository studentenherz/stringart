pub struct PixelLine {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    dx: i32,
    sx: i32,
    dy: i32,
    sy: i32,
    err: i32,
    ended: bool,
}

impl PixelLine {
    pub fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x1 > x0 { 1 } else { -1 };
        let sy = if y1 > y0 { 1 } else { -1 };
        let err = dx + dy;

        Self {
            x0,
            y0,
            x1,
            y1,
            dx,
            sx,
            dy,
            sy,
            err,
            ended: false,
        }
    }
}

impl Iterator for PixelLine {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            return None;
        }
        let res = (self.x0, self.y0);

        let e2 = 2 * self.err;
        if e2 >= self.dy {
            if self.x0 == self.x1 {
                self.ended = true;
            } else {
                self.err += self.dy;
                self.x0 += self.sx;
            }
        }

        if e2 <= self.dx {
            if self.y0 == self.y1 {
                self.ended = true;
            } else {
                self.err += self.dx;
                self.y0 += self.sy;
            }
        }

        Some(res)
    }
}
