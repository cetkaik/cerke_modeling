extern crate lab;
use lab::Lab;

use std::error;
use std::fmt;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

fn dist_sq(a: usize, b: usize, width: usize) -> Option<usize> {
    let (y1, x1) = ((a / width) as isize, (a % width) as isize);
    let (y2, x2) = ((b / width) as isize, (b % width) as isize);
    let delta_y = y1.checked_sub(y2)?;
    let delta_x = x1.checked_sub(x2)?;
    (delta_y.checked_mul(delta_y)? as usize).checked_add(delta_x.checked_mul(delta_x)? as usize)
}

#[derive(Debug, Clone)]
enum ValleyError {
    NoWhitePixel,
    NoBlackPixel,
    Overflow,
    InvalidHeight,
}

impl fmt::Display for ValleyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            ValleyError::NoWhitePixel => write!(f, "no white pixel"),
            ValleyError::NoBlackPixel => write!(f, "no black pixel"),
            ValleyError::Overflow => write!(
                f,
                "image too big; integer overflow occurred while calculating distance."
            ),
            ValleyError::InvalidHeight => write!(f, "invalid height provided"),
        }
    }
}

impl error::Error for ValleyError {
    fn description(&self) -> &str {
        match &self {
            ValleyError::NoWhitePixel => "no white pixel",
            ValleyError::NoBlackPixel => "no black pixel",
            ValleyError::Overflow => {
                "image too big; integer overflow occurred while calculating distance."
            }
            ValleyError::InvalidHeight => "invalid height provided",
        }
    }
}

// square of distance needed from the sea of white pixels to  the most inland black pixel
fn get_maxmin_sqdist(is_black_vec: &[bool], width: usize) -> Result<(usize, Vec<Option<usize>>)> {
    let mut max_min_sqdist = None;
    let mut min_sqdist_vec: Vec<Option<usize>> = vec![None; is_black_vec.len()];
    for (i, is_black) in is_black_vec.iter().enumerate() {
        // for every black pixel
        if !is_black {
            continue;
        }

        let mut minimum_sqdist = None;
        // find the nearest white pixel
        for (j, is_black2) in is_black_vec.iter().enumerate() {
            if *is_black2 {
                continue;
            }

            let sqdist = dist_sq(i, j, width).ok_or_else(|| Box::new(ValleyError::Overflow))?;
            if let Some(c) = minimum_sqdist {
                if c > sqdist {
                    minimum_sqdist = Some(sqdist);
                }
            } else {
                minimum_sqdist = Some(sqdist);
            }
        }

        let minimum_sqdist = minimum_sqdist.ok_or_else(|| Box::new(ValleyError::NoWhitePixel))?;
        min_sqdist_vec[i] = Some(minimum_sqdist);

        if let Some(c) = max_min_sqdist {
            if c < minimum_sqdist {
                max_min_sqdist = Some(minimum_sqdist)
            }
        } else {
            max_min_sqdist = Some(minimum_sqdist)
        }
    }
    let max_min_sqdist = max_min_sqdist.ok_or_else(|| Box::new(ValleyError::NoBlackPixel))?;
    Ok((max_min_sqdist, min_sqdist_vec))
}

// expects 0 <= input <= 1 and should output between 0 and 1.
// for instance, `(2.0 - input) * input` should result in a parabolic valley.
fn get_height_from_disttocoast(input: f64) -> f64 {
    input
}

fn get_color_from_min_sqdist(
    min_sqdist: Option<usize>,
    maxmin_sqdist: usize,
) -> Result<rgb::RGBA8> {
    match min_sqdist {
        None /* white */ => Ok(rgb::RGBA::<u8> {r : 255, g : 255, b : 255, a: 255}),
        Some(sqdist) => {
            let height255 = get_height_from_disttocoast( (sqdist as f64) / (maxmin_sqdist as f64) * 255.0) as i32;
            if height255 < 0 || height255 > 255 {
                return Err(Box::new(ValleyError::InvalidHeight));
            }
            let res = 255 - (height255 as u8);
            Ok( rgb::RGBA::<u8>{ r : res, g: res, b: res, a: 255 })
        }
    }
}

fn convert_and_export(input: lodepng::Bitmap<lodepng::RGBA>, filepath: &str) -> Result<()> {
    let width = input.width;
    let height = input.height;
    let buffer = input.buffer;

    let is_black_vec: Vec<bool> = buffer
        .into_iter()
        .map(|pixel| (Lab::from_rgb(&[pixel.r, pixel.g, pixel.b])).l < 50.0)
        .collect();

    let (maxmin_sqdist, min_sqdist_vec) = get_maxmin_sqdist(&is_black_vec, width)?;

    // maximum distance should give #000000; pixels that are originally white must remain white
    let buffer: Result<Vec<rgb::RGBA<u8>>> = min_sqdist_vec
        .into_iter()
        .map(|min_sqdist| get_color_from_min_sqdist(min_sqdist, maxmin_sqdist))
        .collect();
    let buffer = buffer?;

    lodepng::encode32_file(filepath, &buffer, width, height)?;

    Ok(())
}

fn main() -> Result<()> {
    let image = lodepng::decode32_file("bkauk.png")?;
    convert_and_export(image, "bkauk_valley.png")?;
    Ok(())
}
