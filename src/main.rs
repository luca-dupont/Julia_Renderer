use macroquad::{prelude::*, miniquad::window::screen_size};
extern crate rand;
use num_complex::Complex;
use rayon::prelude::*;

// Define constants
const MAX_ITERS : usize = 500;
const START_BOUNDARY : f64 = 1.5;
const ZOOM_FACTOR: f64 = 1.1;
const SCROLL_FACTOR: f64 = 50.;
const C_RANGE_RE : f64 = 2.;
const C_RANGE_IM : f64 = 0.5;
const TRANSPARENT_GREY: Color = Color{r : 220., g : 220., b : 220., a : 0.2};
const POST_ESCAPE_ITERATIONS : usize = 2;
const FONT_SIZE : f32 = 30.0;
const TEXT_X : f32 = 10.0;
const TEXT_Y : f32 = 30.0;

// Map value from one range to another
fn map_value(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
    // Normalize value to the range [0, 1]
    let normalized_value = (value - from_min) / (from_max - from_min);

    // Scale normalized value to the new range
    normalized_value * (to_max - to_min) + to_min
}

// Function recursively applied to z
fn f(mut z : Complex<f64>, c : Complex<f64>) -> f64 {
    let mut post_iters = 0;
    for i in 0..MAX_ITERS {
        // f_(n+1)(z) = f_(n)(z)^2 + c
        z = z*z + c;
        let norm = z.norm();

        if norm > 2. && post_iters <= POST_ESCAPE_ITERATIONS {
            // Let it run a couple more iterations after it has escaped
            post_iters += 1;
        } else if norm > 2. {
            // Subtract log2(log2(|z|)) from the iterations for smoothness
            return i as f64 - norm.log2().log2()
        }
    }
    // Return the max iterations if it hasn't escaped
    MAX_ITERS as f64
}

#[macroquad::main("Julia Set Simulation")]
async fn main() {
    // Initialize window and image
    let (width, height) = screen_size();

    let mut image = Image::gen_image_color(width as u16, height as u16, BLACK);
    let mut texture = Texture2D::from_image(&image);

    let (mut w, mut h) = (image.width() as f64, image.height() as f64);

    // Last mouse position
    let (mut mx, mut my) = (0.,0.);

    // If the c value changes or not
    let mut freeze = false;

    // Initial graph boundaries
    let mut boundary = START_BOUNDARY;

    // Graph translation offsets
    let mut x_offset = 0.;
    let mut y_offset = 0.;

    loop {
        // Check for window resizing
        let (new_width, new_height) = screen_size();
        if new_width != width || new_height != height {
            image = Image::gen_image_color(new_width as u16, new_height as u16, WHITE);
            texture = Texture2D::from_image(&image);

            (w, h) = (image.width() as f64, image.height() as f64);
        }

        // Check for mouse movement if the screen is not frozen
        let (new_mx, new_my) = mouse_position();
        if (new_mx != mx || new_my != my) && !freeze {
            (mx, my) = (new_mx, new_my);
            // Map the mouse position to the range in which c lies
            (mx, my) = (map_value(mx as f64, 0., w, -C_RANGE_RE, C_RANGE_RE,) as f32, map_value(my as f64, 0., h, -C_RANGE_IM, C_RANGE_IM,) as f32);
        }

        // Handle key presses
        if is_key_down(KeyCode::Equal) {
            // Zoom in
            boundary /= ZOOM_FACTOR;
        }
        if is_key_down(KeyCode::Minus) {
            // Zoom out
            boundary *= ZOOM_FACTOR;
        }
        if is_key_down(KeyCode::Right) {
            // Scroll right
            x_offset += boundary / SCROLL_FACTOR;
        }
        if is_key_down(KeyCode::Left) {
            // Scroll left
            x_offset -= boundary / SCROLL_FACTOR;
        }
        if is_key_down(KeyCode::Down) {
            // Scroll down
            y_offset += boundary / SCROLL_FACTOR;
        }
        if is_key_down(KeyCode::Up) {
            // Scroll up
            y_offset -= boundary / SCROLL_FACTOR;
        }
        if is_key_pressed(KeyCode::Space) {
            // Toggle screen freeze
            freeze = !freeze
        }

        // Initialize c based on the mouse position
        let c = Complex::new(mx as f64, my as f64);

        // Reset background
        clear_background(BLACK);

        // Get a vector of (x,y) pairs for the screen
        let x_y_vec : Vec<(u32, u32)> = (0..w as u32)
            .flat_map(|x| (0..h as u32).map(move |y| (x, y)))
            .collect();

        // Normalize to desired range and map to individual complex values
        let normalized_complex_x_y_vec : Vec<Complex<f64>> = x_y_vec.clone()
            .into_iter()
            .map(|(x,y)| Complex::new(map_value(x as f64, 0., w, -boundary + x_offset, boundary + x_offset, ),map_value(y as f64, 0., h, boundary - y_offset, -boundary - y_offset, )))
            .collect();

        // Apply the f(z) function to each point in parallel
        let iters: Vec<f64> = normalized_complex_x_y_vec
            .par_iter()
            .map(|z| f(*z, c))
            .collect();

        // Color each point depending on their required iterations
        for (iters,( x, y)) in iters.into_iter().zip(x_y_vec.into_iter()) {
            // Map to [0.0, 255.0]
            let iters = map_value(iters, 0., MAX_ITERS as f64, 0., 255.);
            // Invert color
            let iters = 255. - iters as f32;

            let color = Color {r : iters, g : iters, b : iters, a : 1.};
            image.set_pixel(x, y, color);
        }

        // Draw to screen
        texture.update(&image);
        draw_texture(&texture, 0., 0., WHITE);

        // Write the current applied c value
        let c_text = &format!("c = {mx:.3} + {my:.3}i");
        // Draw a semi-transparent rectangle in case the text is over a black section
        let size = measure_text(c_text, None, FONT_SIZE as u16, 1.0);
        draw_rectangle(0.,0.,size.width+TEXT_X, size.height+TEXT_Y, TRANSPARENT_GREY);
        draw_text(c_text, TEXT_X, TEXT_Y, FONT_SIZE, BLACK);

        next_frame().await;
    }
}