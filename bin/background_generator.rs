use chrono::prelude::*;
use image::{GenericImageView, ImageBuffer};
use mtg::db::{get_random_image_uris, init_conn};
use reqwest::blocking::Client;
use std::fs;
use std::fs::File;
use std::io::copy;
use std::{fs::canonicalize, thread::sleep, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut last_ran: i64 = 0;
    let conn = init_conn()?;

    // Reqwest client
    let client = Client::new();

    loop {
        let now = Local::now().timestamp();
        if now - last_ran > 60 * 60 * 12 {
            // Runs every 12 hours
            let image_uris = get_random_image_uris(&conn)?;
            let large = &image_uris.2;
            let full_art = &image_uris.4;

            // Get the card art
            let mut large_image_response = client.get(large).send()?;
            let mut full_art_image_response = client.get(full_art).send()?;
            let mut large_image_file = File::create("large_image.jpg")?;
            let mut full_art_image_file = File::create("full_art_image.jpg")?;
            if large_image_file.metadata()?.len() > 0 || full_art_image_file.metadata()?.len() > 0 {
                // Delete the existing files
                fs::remove_file("large_image.jpg")?;
                fs::remove_file("full_art_image.jpg")?;
            }
            copy(&mut large_image_response, &mut large_image_file)?;
            copy(&mut full_art_image_response, &mut full_art_image_file)?;

            // Load the images
            let portrait = image::open("large_image.jpg")?;
            let mut landscape = image::open("full_art_image.jpg")?;

            // Get dimensions
            let (land_width, land_height) = landscape.dimensions();
            let (port_width, port_height) = portrait.dimensions();

            // Calculate new dimensions for portrait
            let new_port_height = land_height;
            let new_port_width =
                (port_width as f32 * (new_port_height as f32 / port_height as f32)) as u32;

            // Resize portrait
            let resized_portrait = portrait.resize(
                new_port_width,
                new_port_height,
                image::imageops::FilterType::Lanczos3,
            );

            // Blur landscape
            landscape = landscape.blur(5.0);

            // Create a new image buffer
            let mut output = ImageBuffer::new(land_width, land_height);

            // Copy blurred landscape to output
            for (x, y, pixel) in landscape.to_rgba8().enumerate_pixels() {
                output.put_pixel(x, y, *pixel);
            }

            // Calculate position to center portrait
            let x_offset = (land_width - new_port_width) / 2;

            // Overlay portrait on landscape
            for (x, y, pixel) in resized_portrait.to_rgba8().enumerate_pixels() {
                if pixel[3] > 0 {
                    // Only copy non-transparent pixels
                    output.put_pixel(x + x_offset, y, *pixel);
                }
            }

            // Save the result
            output.save("output.png")?;

            let output_full_path = canonicalize("output.png")?;
            wallpaper::set_from_path(output_full_path.to_str().unwrap())?;
            last_ran = now;
        }
        sleep(Duration::from_secs(60));
    }
}
