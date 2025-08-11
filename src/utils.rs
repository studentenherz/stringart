use image::imageops::invert;
use image::{GrayImage, ImageReader, Luma};
use std::collections::HashSet;
use std::f64::consts::PI;
use std::fs::OpenOptions;
use std::io::{Cursor, Write};

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

pub fn generate_stringart(
    image_data: &[u8],
    num_points: usize,
    num_lines: usize,
    weight: u8,
) -> Vec<((i32, i32), (i32, i32))> {
    let mut coordinates: Vec<((i32, i32), (i32, i32))> = vec![];

    let mut img = ImageReader::new(Cursor::new(image_data))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
        .to_luma8();

    invert(&mut img);

    let angles: Vec<f64> = (0..num_points)
        .map(|i| 2.0 * PI * i as f64 / num_points as f64)
        .collect();
    let points: Vec<(f64, f64)> = angles
        .iter()
        .map(|&angle| (angle.cos(), angle.sin()))
        .collect();

    assert!(img.width() == img.height(), "Image has to be sqaure");

    let canvas_size = img.width() as i32;
    let points: Vec<(i32, i32)> = points
        .iter()
        .map(|point| {
            (
                ((point.0 * (canvas_size - 1) as f64) as i32 + canvas_size) / 2,
                ((point.1 * (canvas_size - 1) as f64) as i32 + canvas_size) / 2,
            )
        })
        .collect();

    let mut lines_drawn = HashSet::new();

    let mut last_point_index = 0; // Starting point

    for _ in 0..num_lines {
        let best_next_index = (0..num_points)
            .filter_map(|i| {
                if last_point_index < i && lines_drawn.contains(&(last_point_index, i))
                    || lines_drawn.contains(&(i, last_point_index))
                {
                    return None;
                }

                let p1 = points[i];
                let p2 = points[last_point_index];

                Some((calculate_line_intensity(&img, p1, p2), i))
            })
            .max_by_key(|(intensity, _i)| *intensity)
            .expect(&format!(
                "Can't find a line form point {} that isn't already taken",
                last_point_index
            ))
            .1;

        let p1 = points[last_point_index];
        let p2 = points[best_next_index];
        last_point_index = best_next_index;

        subtract_line(&mut img, p1, p2, weight);

        if last_point_index < best_next_index {
            lines_drawn.insert((last_point_index, best_next_index));
        } else {
            lines_drawn.insert((best_next_index, last_point_index));
        }
        coordinates.push((p1, p2));
    }

    coordinates
}

pub fn export_coordinates(coordinates: &[((i32, i32), (i32, i32))], file_path: &str) {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_path)
        .expect("Failed to open coordinates file");

    for ((x1, y1), (x2, y2)) in coordinates {
        writeln!(&mut file, "{} {} {} {}", x1, y1, x2, y2)
            .expect("Error writing to coordinates file");
    }
}
