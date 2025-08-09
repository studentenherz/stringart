use clap::{builder::styling::AnsiColor, builder::Styles, Parser};
use image::imageops::invert;
use image::{GrayImage, Luma};
use imageproc::drawing::draw_line_segment_mut;
use nalgebra::Vector2;
use rayon::prelude::*;
use std::collections::HashSet;
use std::f64::consts::PI;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use tqdm::tqdm;

mod utils;

use utils::*;

fn cli_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default().bold())
        .usage(AnsiColor::Green.on_default().bold())
        .literal(AnsiColor::Green.on_default().bold())
        .placeholder(AnsiColor::Green.on_default())
}

#[derive(Parser, Debug)]
#[command(
    author = "Michel Romero",
    version,
    about = "Generate string art from a picture",
    long_about = None
)]
#[command(styles=cli_styles())]
struct Cli {
    /// Path to the input image
    input: Option<String>,

    /// Path to save the output image
    #[arg(short, long, default_value = "string_art_output.png")]
    output: String,

    /// Path to save the coordinates of the lines
    #[arg(short, long, default_value = "lines.txt")]
    coordinates: Option<String>,

    /// Number of points
    #[arg(short, long, default_value = "288")]
    points: usize,

    /// Number of lines
    #[arg(short, long, default_value = "4000")]
    lines: usize,

    /// Scale of the image
    #[arg(short, long, default_value = "10.0")]
    scale: f64,

    /// Weight of each line
    #[arg(short, long, default_value = "20")]
    weight: u8,
}

fn main() {
    let cli = Cli::parse();

    let input_path = cli.input;
    let output_path = cli.output;
    let coordinates_path = cli.coordinates;
    let num_points = cli.points;
    let num_lines = cli.lines;
    let scale = cli.scale;
    let weight = cli.weight;

    let mut coordinates: Vec<((i32, i32), (i32, i32))> = vec![];
    let mut canvas_size = 0;

    if let Some(input_path) = input_path {
        let mut file = coordinates_path.map(|coordinates_path| {
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(coordinates_path)
                .expect("Failed to open coordinates file")
        });

        let mut img = image::open(input_path)
            .expect("Failed to open image")
            .to_luma8();
        invert(&mut img);

        let angles: Vec<f64> = (0..num_points)
            .map(|i| 2.0 * PI * i as f64 / num_points as f64)
            .collect();
        let points: Vec<Vector2<f64>> = angles
            .iter()
            .map(|&angle| Vector2::new(angle.cos(), angle.sin()))
            .collect();

        assert!(img.width() == img.height(), "Image has to be sqaure");

        canvas_size = img.width() as i32;
        let points: Vec<(i32, i32)> = points
            .iter()
            .map(|point| {
                (
                    ((point.x * (canvas_size - 1) as f64) as i32 + canvas_size) / 2,
                    ((point.y * (canvas_size - 1) as f64) as i32 + canvas_size) / 2,
                )
            })
            .collect();

        let mut lines_drawn = HashSet::new();

        let mut last_point_index = 0; // Starting point

        for _ in tqdm(0..num_lines) {
            let best_next_index = (0..num_points)
                .into_par_iter()
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

            if let Some(ref mut file) = file {
                writeln!(file, "{} {} {} {}", p1.0, p1.1, p2.0, p2.1)
                    .expect("Error writing to file");
            }

            subtract_line(&mut img, p1, p2, weight);

            if last_point_index < best_next_index {
                lines_drawn.insert((last_point_index, best_next_index));
            } else {
                lines_drawn.insert((best_next_index, last_point_index));
            }
            coordinates.push((p1, p2));
        }
    } else {
        let coordinates_path =
            coordinates_path.expect("If no image is given, this expects a coordinates file");

        let file = fs::File::open(coordinates_path).expect("Failed to open coordinates file");
        for line in io::BufReader::new(file).lines().map_while(Result::ok) {
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.len() == 4 {
                let a: i32 = parts[0].parse().expect("Failed to parse number");
                let b: i32 = parts[1].parse().expect("Failed to parse number");
                let c: i32 = parts[2].parse().expect("Failed to parse number");
                let d: i32 = parts[3].parse().expect("Failed to parse number");

                coordinates.push(((a, b), (c, d)));
                canvas_size = i32::max(canvas_size, a);
                canvas_size = i32::max(canvas_size, b);
                canvas_size = i32::max(canvas_size, c);
                canvas_size = i32::max(canvas_size, d);
            } else {
                eprintln!("Unexpected number of elements in line: {}", line);
            }
        }
    }

    let mut canvas = GrayImage::new(
        (scale * canvas_size as f64) as u32,
        (scale * canvas_size as f64) as u32,
    );
    for pixel in canvas.pixels_mut() {
        *pixel = Luma([255]);
    }

    for (p1, p2) in coordinates {
        let p1 = ((p1.0 as f64 * scale) as i32, (p1.1 as f64 * scale) as i32);
        let p2 = ((p2.0 as f64 * scale) as i32, (p2.1 as f64 * scale) as i32);
        draw_line(&mut canvas, p1, p2);
    }

    canvas.save(output_path).expect("Failed to save image");
}

fn calculate_line_intensity(image: &GrayImage, p1: (i32, i32), p2: (i32, i32)) -> u32 {
    let mut total_intensity = 0u32;

    for (x, y) in PixelLine::new(p1.0, p1.1, p2.0, p2.1) {
        let pixel_value = image.get_pixel(x as u32, y as u32).0[0];
        total_intensity += pixel_value as u32;
    }

    total_intensity
}

fn draw_line(image: &mut GrayImage, p1: (i32, i32), p2: (i32, i32)) {
    draw_line_segment_mut(
        image,
        (p1.0 as f32, p1.1 as f32),
        (p2.0 as f32, p2.1 as f32),
        Luma([0]),
    );
}

fn subtract_line(image: &mut GrayImage, p1: (i32, i32), p2: (i32, i32), weight: u8) {
    for (x, y) in PixelLine::new(p1.0, p1.1, p2.0, p2.1) {
        let pixel = image.get_pixel_mut(x as u32, y as u32);
        *pixel = Luma([pixel.0[0].saturating_sub(weight)]);
    }
}
