use okbayer::dither_bayer_oklab;
use imagequant::{Attributes, RGBA};

pub fn palette_from_image(image_rgb8: Vec<u8>, width: u32, height: u32, palette_size: u32) -> Vec<u8> {
    let expected_len = (width as usize) * (height as usize) * 3;
    if image_rgb8.len() != expected_len {
        return Vec::new();
    }

    let pixel_count = (width as usize) * (height as usize);
    let mut rgba_pixels: Vec<RGBA> = Vec::with_capacity(pixel_count);

    for chunk in image_rgb8.chunks_exact(3) {
        rgba_pixels.push(RGBA {
            r: chunk[0],
            g: chunk[1],
            b: chunk[2],
            a: 255,
        });
    }

    let mut attr = Attributes::new();
    let valid_palette_size = palette_size.min(256).max(1) as u32;
    let _ = attr.set_max_colors(valid_palette_size);
    let _ = attr.set_speed(1); 

    let mut image_wrapper = attr
        .new_image(rgba_pixels, width as usize, height as usize, 0.0)
        .expect("Failed to create image wrapper");

    let mut result = match attr.quantize(&mut image_wrapper) {
        Ok(res) => res,
        Err(_) => return Vec::new(),
    };

    let palette = result.palette();
    
    let mut flat_palette: Vec<u8> = Vec::with_capacity((palette_size as usize) * 3);
    
    for i in 0..palette_size as usize {
        if i < palette.len() {
            let color = palette[i];
            flat_palette.push(color.r);
            flat_palette.push(color.g);
            flat_palette.push(color.b);
        } else {
            flat_palette.push(0);
            flat_palette.push(0);
            flat_palette.push(0);
        }
    }

    flat_palette
}

fn hex_str_to_u8(hex_str: &str) -> u8 {
    u8::from_str_radix(hex_str, 16).unwrap()
}

fn parse_hex_color(hex: &str) -> (u8, u8, u8) {
    if hex.len() != 6 {
        panic!("Invalid hex color: {}, expected 6 characters", hex);
    }
    let r_hex = &hex[0..2];
    let g_hex = &hex[2..4];
    let b_hex = &hex[4..6];
    let r = hex_str_to_u8(r_hex);
    let g = hex_str_to_u8(g_hex);
    let b = hex_str_to_u8(b_hex);
    (r, g, b)
}

fn palette_from_string(palette_string: &str) -> Vec<u8> {
    // parse each hex color into RGB bytes
    let mut palette: Vec<u8> = Vec::new();
    for line in palette_string.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (r, g, b) = parse_hex_color(line);
        palette.push(r);
        palette.push(g);
        palette.push(b);
    }
    palette
}  

fn main() {
    let image_data = include_bytes!("../images/flower.png");
    let image = image::load_from_memory(image_data).unwrap();
    let rgb_image = image.to_rgb8();
    let image_bytes = rgb_image.as_raw();
    let width = rgb_image.width();
    let height = rgb_image.height();

    let palette = include_str!("../palettes/island-joy-16.hex");
    let palette_bytes = palette_from_string(palette);

    let dithered_bytes = dither_bayer_oklab(image_bytes, &palette_bytes, width, height, 1.0).unwrap();
    let dithered_image = image::RgbImage::from_raw(width, height, dithered_bytes).unwrap();
    dithered_image.save("images/dithered_flower_0.0.png").unwrap();

    // imagequant
    let palettes: Vec<Vec<u8>> = vec![
        palette_from_image(image_bytes.to_vec(), width, height, 2),
        palette_from_image(image_bytes.to_vec(), width, height, 9),
        palette_from_image(image_bytes.to_vec(), width, height, 10),
        palette_from_image(image_bytes.to_vec(), width, height, 16),
        palette_from_image(image_bytes.to_vec(), width, height, 32),
        palette_from_image(image_bytes.to_vec(), width, height, 256),
    ];
    for (i, palette) in palettes.iter().enumerate() {
        let dithered_bytes = dither_bayer_oklab(&image_bytes, &palette, width, height, 1.0).unwrap();
        let dithered_image = image::RgbImage::from_raw(width, height, dithered_bytes).unwrap();
        dithered_image.save(&format!("images/dithered_flower_1.0_{}_imagequant.png", i)).unwrap();
    }

}