use image::{GrayImage, Luma};

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

pub fn calculate_line_intensity(image: &GrayImage, p1: (i32, i32), p2: (i32, i32)) -> u32 {
    let mut total_intensity = 0u32;

    for (x, y) in PixelLine::new(p1.0, p1.1, p2.0, p2.1) {
        let pixel_value = image.get_pixel(x as u32, y as u32).0[0];
        total_intensity += pixel_value as u32;
    }

    total_intensity
}

pub fn draw_line(image: &mut GrayImage, p1: (i32, i32), p2: (i32, i32)) {
    for (x, y) in PixelLine::new(p1.0, p1.1, p2.0, p2.1) {
        let pixel = image.get_pixel_mut(x as u32, y as u32);
        *pixel = Luma([0]);
    }
}

pub fn subtract_line(image: &mut GrayImage, p1: (i32, i32), p2: (i32, i32), weight: u8) {
    for (x, y) in PixelLine::new(p1.0, p1.1, p2.0, p2.1) {
        let pixel = image.get_pixel_mut(x as u32, y as u32);
        *pixel = Luma([pixel.0[0].saturating_sub(weight)]);
    }
}
