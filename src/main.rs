use clap::{builder::styling::AnsiColor, builder::Styles, Parser};
use image::{GrayImage, Luma};
use std::fs;
use std::io::{self, BufRead};

mod stringart;
pub mod utils;

use stringart::*;
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
    let scale = cli.scale;
    let num_lines = cli.lines;
    let weight = cli.weight;

    let mut canvas_size = 0;

    let coordinates = if let Some(input_path) = input_path {
        let image_data = fs::read(&input_path).expect("Error reading the input file!");

        let coordinates = generate_stringart(&image_data, num_points, num_lines, weight);
        if let Some(file_path) = coordinates_path {
            export_coordinates(&coordinates, &file_path);
        }

        coordinates
    } else {
        let mut coordinates: Vec<((i32, i32), (i32, i32))> = vec![];
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
            } else {
                eprintln!("Unexpected number of elements in line: {}", line);
            }
        }

        coordinates
    };

    for ((a, b), (c, d)) in coordinates.iter() {
        canvas_size = i32::max(canvas_size, *a);
        canvas_size = i32::max(canvas_size, *b);
        canvas_size = i32::max(canvas_size, *c);
        canvas_size = i32::max(canvas_size, *d);
    }
    canvas_size += 1;

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
