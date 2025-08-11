use image::imageops::invert;
use image::ImageReader;
use std::collections::HashSet;
use std::f64::consts::PI;
use std::io::Cursor;

use crate::utils::{calculate_line_intensity, subtract_line};

#[allow(dead_code)]
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
