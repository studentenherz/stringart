use clap::Parser;
use image::imageops::invert;
use image::{GrayImage, Luma};
use imageproc::drawing::draw_line_segment_mut;
use nalgebra::Vector2;
use std::cmp::{max, min};
use std::collections::HashSet;
use std::f64::consts::PI;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use std::mem::swap;
use tqdm::tqdm;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the input image
    #[arg(short, long)]
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

    /// scale
    #[arg(short, long, default_value = "10.0")]
    scale: f64,

    /// scale
    #[arg(short, long, default_value = "20")]
    weight: u8,
}

fn main() {
    let args = Args::parse();

    let input_path = args.input;
    let output_path = args.output;
    let coordinates_path = args.coordinates;
    let num_points = args.points;
    let num_lines = args.lines;
    let scale = args.scale;
    let weight = args.weight;

    let mut coordinates: Vec<((i32, i32), (i32, i32))> = vec![];
    let mut canvas_size = 0.0;

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

        canvas_size = img.width() as f64;
        let points: Vec<(i32, i32)> = points
            .iter()
            .map(|point| {
                (
                    ((point.x + 1.0) * (canvas_size / 2.0)) as i32,
                    ((point.y + 1.0) * (canvas_size / 2.0)) as i32,
                )
            })
            .collect();

        let mut lines_drawn = HashSet::new();

        for _ in tqdm(0..num_lines) {
            let mut max_value = 0.0;
            let mut best_pair = (0, 0);

            for i in 0..num_points {
                for j in (i + 1)..num_points {
                    if lines_drawn.contains(&(i, j)) {
                        continue;
                    }

                    let p1 = points[i];
                    let p2 = points[j];
                    let line_intensity = calculate_line_intensity(&img, p1, p2);
                    if line_intensity > max_value {
                        max_value = line_intensity;
                        best_pair = (i, j);
                    }
                }
            }

            let p1 = points[best_pair.0];
            let p2 = points[best_pair.1];

            if let Some(ref mut file) = file {
                writeln!(file, "{} {} {} {}", p1.0, p1.1, p2.0, p2.1)
                    .expect("Error writing to file");
            }

            subtract_line(&mut img, p1, p2, weight);

            lines_drawn.insert(best_pair);
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
                canvas_size = f64::max(canvas_size, a as f64);
                canvas_size = f64::max(canvas_size, b as f64);
                canvas_size = f64::max(canvas_size, c as f64);
                canvas_size = f64::max(canvas_size, d as f64);
            } else {
                eprintln!("Unexpected number of elements in line: {}", line);
            }
        }
    }

    let mut canvas = GrayImage::new((scale * canvas_size) as u32, (scale * canvas_size) as u32);
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

fn calculate_line_intensity(image: &GrayImage, p1: (i32, i32), p2: (i32, i32)) -> f64 {
    let mut total_intensity = 0.0;
    let mut count = 0;
    let (mut x1, mut y1) = (p1.0 as f64, p1.1 as f64);
    let (mut x2, mut y2) = (p2.0 as f64, p2.1 as f64);
    let mut dx = x2 - x1;
    let mut dy = y2 - y1;

    if dx.abs() > dy.abs() {
        let (mut x1, mut x2) = (p1.0, p2.0);
        if x1 > x2 {
            swap(&mut x1, &mut x2);
            swap(&mut y1, &mut y2);
            dx *= -1.0;
            dy *= -1.0;
        }
        for x in x1..=x2 {
            let y = y1 + dy * (x - x1) as f64 / dx;
            let x = max(0, min(x as u32, image.width() - 1));
            let y = max(0, min(y.round() as u32, image.height() - 1));
            let pixel_value = image.get_pixel(x, y).0[0];
            total_intensity += pixel_value as f64;
            count += 1;
        }
    } else {
        let (mut y1, mut y2) = (p1.1, p2.1);
        if y1 > y2 {
            swap(&mut x1, &mut x2);
            swap(&mut y1, &mut y2);
            dx *= -1.0;
            dy *= -1.0;
        }
        for y in y1..=y2 {
            let x = x1 + dx * (y - y1) as f64 / dy;
            let x = max(0, min(x.round() as u32, image.width() - 1));
            let y = max(0, min(y as u32, image.height() - 1));
            let pixel_value = image.get_pixel(x, y).0[0];
            total_intensity += pixel_value as f64;
            count += 1;
        }
    }

    total_intensity / count as f64
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
    let canvas_size = image.width();
    let mut mask = GrayImage::new(canvas_size, canvas_size);
    draw_line_segment_mut(
        &mut mask,
        (p1.0 as f32, p1.1 as f32),
        (p2.0 as f32, p2.1 as f32),
        Luma([weight]),
    );

    for i in 0..image.width() {
        for j in 0..image.height() {
            let pixel = image.get_pixel_mut(i, j);
            *pixel = Luma([pixel.0[0].saturating_sub(mask.get_pixel(i, j).0[0])]);
        }
    }
}
