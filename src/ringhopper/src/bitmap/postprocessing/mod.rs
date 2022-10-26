use crate::types::{ColorARGBInt, log2_u16, ColorARGB, ColorRGB, Vector3D, Point2D, ColorRGBFn};

use super::*;

/// Result of a processed bitmap.
#[derive(Clone)]
pub struct ProcessedBitmap {
    /// All pixels of the bitmap.
    pub pixels: Vec<ColorARGBInt>,

    /// Height of the bitmap in pixels.
    pub height: usize,

    /// Width of the bitmap in pixels.
    pub width: usize,

    /// Depth of the bitmap in pixels.
    pub depth: usize,

    /// Number of mipmaps in the bitmap.
    pub mipmaps: usize,

    /// Number of faces in the bitmap.
    ///
    /// For cubemaps, this is 6. For all other bitmaps, this is 1.
    pub faces: usize,

    /// Registration point
    pub registration_point: Point2D,

    /// Floating point versions of the pixels.
    pixels_float: Vec<ColorARGB>
}

/// Algorithm to use for bumpmap generation
#[derive(Copy, Clone, Default, PartialEq)]
pub enum BumpmapAlgorithm {
    /// Simple algorithm that uses the delta of adjacent pixels.
    ///
    /// This is most similar to Halo: Combat Evolved's bumpmap generator.
    #[default]
    Fast,

    /// Uses the Sobel operator, resulting in a much smoother heightmap.
    Sobel
}

/// Detail fade color to use.
#[derive(Copy, Clone, Default, PartialEq)]
pub enum DetailFadeColor {
    /// Fade to #7F7F7F gray.
    #[default]
    Gray,

    /// Fade to the average color of the bitmap.
    Average
}

/// Options for configuring the bitmap processor.
#[derive(Copy, Clone, Default)]
pub struct ProcessingOptions {
    /// If `Some`, convert a monochrome input into a bumpmap, setting the height to make the bumpmap.
    pub bumpmap_height: Option<f64>,

    /// Algorithm to use for bumpmaps.
    pub bumpmap_algorithm: BumpmapAlgorithm,

    /// If `true`, use gamma correction when generating mipmaps.
    pub gamma_corrected_mipmaps: bool,

    /// If `Some`, fade mipmaps to gray by a factor after doing mipmap generation.
    pub detail_fade_factor: Option<f64>,

    /// If `Some`, sharpen the base map and mipmaps by a factor after doing mipmap generation.
    pub sharpen_factor: Option<f64>,

    /// If `Some`, blur the base map by a factor before doing mipmap generation.
    pub blur_factor: Option<f64>,

    /// If `Some`, modify the alpha by this factor after doing mipmap generation.
    pub alpha_bias: Option<f64>,

    /// If `Some`, limit the maximum number of mipmaps to generate.
    pub max_mipmaps: Option<usize>,

    /// If `true`, truncate pixels with no alpha.
    pub truncate_zero_alpha: bool,

    /// If `true`, vectorize each pixel of the final result.
    pub vectorize: bool,

    /// If `true`, use nearest neighbor scaling for the alpha channel.
    pub nearest_neighbor_alpha_mipmap: bool,

    /// Fade color to use.
    pub detail_fade_color: DetailFadeColor
}

/// Final processed bitmap sequence.
#[derive(Clone, Default)]
pub struct ProcessedSequence {
    /// First bitmap in the sequence, if set.
    pub first_bitmap: Option<usize>,

    /// Number of bitmaps in the sequence.
    pub bitmap_count: usize,

    /// Sprites, if present.
    pub sprites: Vec<Sprite>
}

/// Bitmap postprocessor which can generate mipmaps and do postprocessing.
#[derive(Clone)]
pub struct ProcessedBitmaps {
    /// All final sequences.
    pub sequences: Vec<ProcessedSequence>,

    /// All processed bitmaps.
    pub bitmaps: Vec<ProcessedBitmap>,

    /// Options used to process the bitmaps.
    options: ProcessingOptions,

    /// Input type for the color plate.
    color_plate_type: ColorPlateInputType
}

// Wrap around the dimensions
const fn wrap_around(x: usize, y: usize, xd: isize, yd: isize, width: usize, height: usize) -> (usize,usize) {
    let iwidth = width as isize;
    let iheight = height as isize;

    let mut x_after = (x as isize + xd) % iwidth;
    if x_after < 0 {
        x_after += iwidth;
    }
    let mut y_after = (y as isize + yd) % iheight;
    if y_after < 0 {
        y_after += iheight;
    }

    (x_after as usize, y_after as usize)
}

macro_rules! iterate_mipmaps {
    ($bitmap:expr, $function:expr) => {{
        if $bitmap.mipmaps > 0 {
            iterate_base_map_and_mipmaps(($bitmap.width / 2).max(1), ($bitmap.height / 2).max(1), 1, 1, $bitmap.mipmaps - 1, $function)
        }
    }}
}

impl ProcessedBitmaps {
    /// Process the color plate, returning a final set of bitmaps.
    pub fn process_color_plate(color_plate: ColorPlate, options: &ProcessingOptions) -> ProcessedBitmaps {
        // Load the processed bitmaps.
        let mut processed_bitmaps = ProcessedBitmaps::load_color_plate(color_plate, options);

        processed_bitmaps.perform_blur();
        processed_bitmaps.generate_heightmaps();
        processed_bitmaps.generate_mipmaps();
        processed_bitmaps.detail_fade();
        processed_bitmaps.perform_sharpen();
        processed_bitmaps.truncate_zero_alpha();
        processed_bitmaps.alpha_bias();
        processed_bitmaps.vectorize();
        processed_bitmaps.consolidate_textures();

        // Finally, convert to integer RGB
        for i in &mut processed_bitmaps.bitmaps {
            i.pixels.reserve_exact(i.pixels_float.len());
            for p in &i.pixels_float {
                i.pixels.push((*p).into());
            }
            i.pixels_float = Vec::new();
        }

        processed_bitmaps
    }

    /// Use the color plate data to make a set of processed bitmaps.
    fn load_color_plate(color_plate: ColorPlate, options: &ProcessingOptions) -> ProcessedBitmaps {
        let mut processed_bitmaps = ProcessedBitmaps {
            sequences: Vec::new(),
            bitmaps: Vec::new(),
            options: *options,
            color_plate_type: color_plate.options.input_type
        };

        processed_bitmaps.bitmaps.reserve_exact(color_plate.bitmaps.len());
        processed_bitmaps.sequences.reserve_exact(color_plate.sequences.len());

        // Copy in the bitmaps.
        for b in color_plate.bitmaps {
            let mut pixels_float = Vec::with_capacity(b.pixels.len());
            for p in b.pixels {
                pixels_float.push(p.into());
            }
            processed_bitmaps.bitmaps.push(ProcessedBitmap { pixels: Vec::new(), height: b.height, width: b.width, depth: 1, mipmaps: 0, faces: 1, pixels_float, registration_point: b.registration_point })
        }

        // Copy in the sequences.
        for s in color_plate.sequences {
            processed_bitmaps.sequences.push(ProcessedSequence { first_bitmap: s.first_bitmap, bitmap_count: s.bitmap_count, sprites: s.sprites })
        }

        processed_bitmaps
    }

    /// If a bitmap has zero alpha, set to black. (alpha blend usage)
    ///
    /// NOTE: This function is very broken and has two issues:
    /// 1. It resizes bitmaps on load, effectively treating 0 alpha pixels as a second dummy space.
    /// 2. Postprocessing is only done on mipmaps, not the base map as seen with effects\particles\solid\bitmaps\panel debris.bitmap
    fn truncate_zero_alpha(&mut self) {
        const BLACK: ColorARGB = ColorARGB { a: 0.0, r: 0.0, g: 0.0, b: 0.0 };

        if !self.options.truncate_zero_alpha {
            return
        }

        for b in &mut self.bitmaps {
            let pixels_float = &mut b.pixels_float[b.width * b.height * b.depth * b.faces..];
            iterate_mipmaps!(b, |m| {
                for p in &mut pixels_float[m.pixel_offset..m.pixel_offset+m.size] {
                    if p.a == 0.0 {
                        *p = BLACK;
                    }
                }
            })
        }
    }

    /// Perform a blur. This factors in gamma compression.
    fn perform_blur(&mut self) {
        let blur = match self.options.blur_factor {
            Some(it) => it,
            _ => return,
        };

        // Apply blur to the base map.
        for b in &mut self.bitmaps {
            let mut output = Vec::<ColorARGB>::with_capacity(b.pixels_float.len());

            let blur = blur / 2.0;
            let max_distance = blur + 1.0;
            let max_distance_squared = (max_distance * max_distance) as f32;
            let max_distance_usize = max_distance as usize;

            for y in 0..b.height {
                for x in 0..b.width {
                    let point = Point2D { x: x as f32, y: y as f32 };

                    let mut total_color = ColorARGB::default();
                    let mut total_factor = 0.0;

                    for y2 in 0..1+max_distance_usize*2 {
                        for x2 in 0..1+max_distance_usize*2 {
                            let x2_delta = (x2 as isize) - max_distance_usize as isize;
                            let y2_delta = (y2 as isize) - max_distance_usize as isize;
                            let (real_x2, real_y2) = wrap_around(x, y, x2_delta, y2_delta, b.width, b.height);

                            let point_c = Point2D { x: (x as isize + x2_delta) as f32, y: (y as isize + y2_delta) as f32 };
                            let distance_squared = point_c.distance_from_point_squared(&point);
                            if distance_squared >= max_distance_squared {
                                continue;
                            }

                            // We use gamma_decompress because blurring ends up creating nasty artifacts if we don't do this.
                            //
                            // See https://www.youtube.com/watch?v=LKnqECcg6Gw for a cool video on this.
                            let color = b.pixels_float[real_x2 + real_y2 * b.width].gamma_decompress();
                            let factor = 1.0 - (distance_squared / max_distance_squared).sqrt();

                            total_factor += factor;
                            total_color.a += color.a * factor;
                            total_color.r += color.r * factor;
                            total_color.g += color.g * factor;
                            total_color.b += color.b * factor;
                        }
                    }

                    total_color.a = total_color.a / total_factor;
                    total_color.r = total_color.r / total_factor;
                    total_color.g = total_color.g / total_factor;
                    total_color.b = total_color.b / total_factor;
                    output.push(total_color.gamma_compress());
                }
            }

            for i in 0..output.len() {
                b.pixels_float[i] = output[i];
            }
        }
    }

    fn perform_sharpen(&mut self) {
        let sharpen_factor = match self.options.sharpen_factor {
            Some(it) => it.clamp(0.0, 1.0),
            _ => return,
        };

        // Sharpening is just value + factor / (1 - factor) * average difference of the 8 adjacent pixels, wrapping if needed
        let sharpen_multiplier = (sharpen_factor / (1.0 - sharpen_factor)) as f32;
        for b in &mut self.bitmaps {
            let mut new_pixels = Vec::with_capacity(b.pixels_float.len());

            iterate_base_map_and_mipmaps(b.width, b.height, b.depth, b.faces, b.mipmaps, |m| {
                let pixels = &b.pixels_float[m.pixel_offset .. m.pixel_offset + m.size];

                // Don't sharpen mipmaps that are too small
                if m.height <= 2 || m.width <= 2 {
                    new_pixels.extend_from_slice(&pixels);
                    return;
                }

                let face_size = m.size / (m.depth * b.faces);

                for p in (0..pixels.len()).step_by(face_size) {
                    let face = &pixels[p..p+face_size];

                    for y in 0..m.height {
                        for x in 0..m.width {
                            let (left, top) = wrap_around(x, y, -1, -1, m.width, m.height);
                            let (right, bottom) = wrap_around(x, y, 1, 1, m.width, m.height);

                            let mut total = ColorRGB::default();
                            let pixel = pixels[p + x+y*m.width];

                            macro_rules! add_pixel {
                                ($x:expr, $y:expr) => {{
                                    let pixel_to_check = face[$x + $y * m.width];
                                    total.r += pixel_to_check.r;
                                    total.g += pixel_to_check.g;
                                    total.b += pixel_to_check.b;
                                }}
                            }

                            add_pixel!(left, top);
                            add_pixel!(left, y);
                            add_pixel!(left, bottom);

                            add_pixel!(x, top);
                            // don't add center pixel
                            add_pixel!(x, bottom);

                            add_pixel!(right, top);
                            add_pixel!(right, y);
                            add_pixel!(right, bottom);

                            total.r /= 8.0;
                            total.g /= 8.0;
                            total.b /= 8.0;

                            let sharpened_pixel = ColorRGB {
                                r: (pixel.r + (pixel.r - total.r) * sharpen_multiplier).clamp(0.0, 1.0),
                                g: (pixel.g + (pixel.g - total.g) * sharpen_multiplier).clamp(0.0, 1.0),
                                b: (pixel.b + (pixel.b - total.b) * sharpen_multiplier).clamp(0.0, 1.0)
                            };

                            new_pixels.push(ColorARGB::from_rgb(pixel.a, sharpened_pixel));
                        }
                    }
                }
            });

            b.pixels_float = new_pixels;
        }
    }

    /// Consolidate cubemaps and 3D textures into one texture.
    fn consolidate_textures(&mut self) {
        match self.color_plate_type {
            ColorPlateInputType::ThreeDimensionalTextures => (),
            ColorPlateInputType::Cubemaps => (),
            _ => return
        }

        let mut new_bitmaps = Vec::new();
        let mut new_sequences = Vec::with_capacity(self.sequences.len());

        for s in &self.sequences {
            if s.first_bitmap.is_none() {
                new_sequences.push(s.clone());
                continue;
            }

            let first_bitmap_index = s.first_bitmap.unwrap();
            let first_bitmap = &self.bitmaps[first_bitmap_index];
            let width = first_bitmap.width;
            let height = first_bitmap.height;
            let (depth, faces) = match self.color_plate_type {
                ColorPlateInputType::Cubemaps => (1, s.bitmap_count),
                ColorPlateInputType::ThreeDimensionalTextures => (s.bitmap_count, 1),
                _ => unreachable!()
            };
            let mipmaps = first_bitmap.mipmaps;

            let mut capacity = 0;
            for b in first_bitmap_index..first_bitmap_index+s.bitmap_count {
                capacity += self.bitmaps[b].pixels_float.len();
            }

            let mut new_bitmap = ProcessedBitmap {
                pixels: Vec::new(),
                height,
                width,
                depth,
                mipmaps,
                faces,
                pixels_float: Vec::with_capacity(capacity),
                registration_point: Point2D::default() // registration point is dropped for cubemaps and 3D textures
            };
            iterate_base_map_and_mipmaps(width, height, 1, 1, mipmaps, |m| {
                for b in &self.bitmaps[first_bitmap_index..first_bitmap_index+s.bitmap_count] {
                    new_bitmap.pixels_float.extend_from_slice(&b.pixels_float[m.pixel_offset..m.pixel_offset+m.size]);
                }
            });

            new_sequences.push(ProcessedSequence { first_bitmap: Some(new_bitmaps.len()), bitmap_count: 1, sprites: s.sprites.clone() });
            new_bitmaps.push(new_bitmap);
        }

        self.bitmaps = new_bitmaps;
        self.sequences = new_sequences;

        // Fix 3D texture mipmaps, since depth is also mipmapped too.
        for b in &mut self.bitmaps {
            if b.depth == 1 {
                continue;
            }

            // NOTE: Deliberately passing depth as faces since it is stored like that and we want to make it store it correctly.
            //
            // You should not do this (outside of this function).
            let mut new_pixels = Vec::<ColorARGB>::with_capacity(b.pixels_float.len());
            iterate_base_map_and_mipmaps(b.width, b.height, 1, b.depth, b.mipmaps, |m| {
                // For the first index, just passthrough it.
                let mipmap_size = m.size / b.depth;
                let compaction = 1 << m.index;
                let depth = (b.depth / compaction).max(1);

                for i in 0..depth {
                    let first_level = i * compaction;
                    let end = (i + 1) * compaction;

                    let mut new_pixels_buffer = Vec::<ColorARGB>::with_capacity(mipmap_size);

                    for p in 0..mipmap_size {
                        let mut new_pixel = ColorARGB::default();

                        for l in first_level..end {
                            let adding_pixel = &b.pixels_float[m.pixel_offset + l * mipmap_size + p];
                            new_pixel.a += adding_pixel.a;
                            new_pixel.r += adding_pixel.r;
                            new_pixel.g += adding_pixel.g;
                            new_pixel.b += adding_pixel.b;
                        }

                        new_pixel.a /= compaction as f32;
                        new_pixel.r /= compaction as f32;
                        new_pixel.g /= compaction as f32;
                        new_pixel.b /= compaction as f32;

                        new_pixels_buffer.push(new_pixel);
                    }

                    new_pixels.append(&mut new_pixels_buffer);
                }
            });

            b.pixels_float = new_pixels;
        }
    }

    /// Vectorize bitmaps.
    fn vectorize(&mut self) {
        if !self.options.vectorize {
            return;
        }

        for b in &mut self.bitmaps {
            for p in &mut b.pixels_float {
                *p = p.vector_normalize();
            }
        }
    }

    /// Generate heightmaps from monochrome input.
    fn generate_heightmaps(&mut self) {
        if self.options.bumpmap_height.is_none() {
            return;
        }

        let h = self.options.bumpmap_height.unwrap() as f32;

        for b in &mut self.bitmaps {
            let mut new_pixels = Vec::with_capacity(b.pixels_float.len());

            for center_y in 0..b.height {
                let top_y = center_y.saturating_sub(1);
                let bottom_y = (center_y + 1).min(b.height - 1);

                for center_x in 0..b.width {
                    let left_x = center_x.saturating_sub(1);
                    let right_x = (center_x + 1).min(b.width - 1);

                    let strength = |x,y| {
                        let pixel: &ColorARGB = &b.pixels_float[x + y * b.width];
                        pixel.r * 2.0 - 1.0
                    };

                    let center = strength(center_x, center_y);
                    let top = strength(center_x, top_y);
                    let left = strength(left_x, center_y);
                    let right = strength(right_x, center_y);
                    let bottom = strength(center_x, bottom_y);

                    let (dx, dy, dz);

                    match self.options.bumpmap_algorithm {
                        BumpmapAlgorithm::Fast => {
                            dx = (left - center).max(0.0) - (right - center).max(0.0);
                            dy = (top - center).max(0.0) - (bottom - center).max(0.0);
                            dz = 1.0 / (h * 150.0);
                        },
                        BumpmapAlgorithm::Sobel => {
                            let top_right = strength(right_x, top_y);
                            let top_left = strength(left_x, top_y);
                            let bottom_left = strength(left_x, bottom_y);
                            let bottom_right = strength(right_x, bottom_y);

                            // from https://stackoverflow.com/questions/2368728/can-normal-maps-be-generated-from-a-texture/2368794#2368794
                            dx = -((top_right + 2.0 * right + bottom_right) - (top_left + 2.0 * left + bottom_left));
                            dy = -((bottom_left + 2.0 * bottom + bottom_right) - (top_left + 2.0 * top + top_right));
                            dz = 1.0 / (h * 20.0);
                        }
                    }

                    let v = Vector3D { x: dx, y: dy, z: dz }.normalize();
                    let c = ColorARGB {
                        a: b.pixels_float[center_x + center_y * b.width].a,
                        r: v.x / 2.0 + 0.5,
                        g: v.y / 2.0 + 0.5,
                        b: v.z / 2.0 + 0.5
                    };

                    new_pixels.push(c);
                }
            }

            debug_assert_eq!(b.pixels_float.len(), new_pixels.len());
            b.pixels_float = new_pixels;
        }
    }

    /// Do detail fade on mipmaps.
    fn detail_fade(&mut self) {
        let fade = match self.options.detail_fade_factor {
            Some(it) => it,
            _ => return,
        };

        for bitmap in &mut self.bitmaps {
            let (base_map_pixels, mipmap_pixels) = bitmap.pixels_float.split_at_mut(bitmap.width * bitmap.height);

            if bitmap.mipmaps > 0 {
                let fade = fade.clamp(0.0, 1.0) as f32;
                let mipmap_count_float = bitmap.mipmaps as f32;
                let overall_fade_factor = mipmap_count_float - fade * (mipmap_count_float - 1.0 + (1.0 - fade));

                // Get the fade color
                let (r,g,b) = match self.options.detail_fade_color {
                    DetailFadeColor::Gray => (127.0 / 255.0, 127.0 / 255.0, 127.0 / 255.0),
                    DetailFadeColor::Average => {
                        let mut average = ColorRGB::default();
                        for p in &base_map_pixels[..] {
                            average.r += p.r;
                            average.g += p.g;
                            average.b += p.b;
                        }
                        let sum = base_map_pixels.len() as f32;
                        (average.r / sum, average.g / sum, average.b / sum)
                    }
                };

                // Do it!
                iterate_mipmaps!(bitmap, |m| {
                    // The amount we fade depends on the mipmap. We can use alpha blending to calculate this.
                    let fade_to_gray = ColorARGB {
                        a: (((m.index + 1) as f32) / overall_fade_factor).min(1.0),
                        r,
                        g,
                        b
                    };
                    for px in &mut mipmap_pixels[m.pixel_offset..m.pixel_offset + m.size] {
                        *px = ColorARGB::from_rgb(px.a, ColorARGB::from_rgb(1.0, px.rgb()).alpha_blend(fade_to_gray).rgb());
                    }
                });
            }
        }
    }

    /// Do alpha bias on mipmaps.
    fn alpha_bias(&mut self) {
        let alpha_bias = match self.options.alpha_bias {
            Some(it) => it,
            _ => return,
        };

        for b in &mut self.bitmaps {
            let mipmap_pixels = &mut b.pixels_float[b.width * b.height..];

            let alpha_bias = alpha_bias.clamp(-1.0, 1.0) as f32;
            let mipmap_count_float = b.mipmaps as f32;

            iterate_mipmaps!(b, |m| {
                let delta = alpha_bias * ((m.index + 1) as f32) / mipmap_count_float;
                for p in &mut mipmap_pixels[m.pixel_offset..m.pixel_offset + m.size] {
                    p.a = (p.a + delta).clamp(0.0, 1.0);
                }
            });
        }
    }

    /// Generate mipmaps.
    fn generate_mipmaps(&mut self) {
        let gamma_corrected_mipmaps = self.options.gamma_corrected_mipmaps;
        let nearest_neighbor_alpha_mipmap = self.options.nearest_neighbor_alpha_mipmap;

        for b in &mut self.bitmaps {
            // Get the highest dimension.
            //
            // Mipmap count is log2 this.
            //
            // For example, a 32x8 texture will be mipmapped like this:
            //
            // - 32x8 (base map)
            // - 16x4 (mipmap 1)
            // -  8x2 (mipmap 2)
            // -  4x1 (mipmap 3)
            // -  2x1 (mipmap 4)
            // -  1x1 (mipmap 5)
            //
            // log2(32) = 5
            //
            //
            // NOTE: Non-power-of-two textures are rounded down.
            //
            // For example, a 36x9 texture will be mipmapped like this.
            //
            // - 36x9 (base map)
            // - 18x4 (mipmap 1)
            // -  9x2 (mipmap 2)
            // -  4x1 (mipmap 3)
            // -  2x1 (mipmap 4)
            // -  1x1 (mipmap 5)
            //
            // log2(36) = 5.169925... which rounds down to 5
            //
            // 3D textures are treated the same, which we will generate the final 3D texture mipmaps later.
            let maximum_mipmap_count = log2_u16(b.height.max(b.width) as u16) as usize;

            // Override this.
            let final_mipmap_count = if let Some(n) = self.options.max_mipmaps {
                maximum_mipmap_count.min(n)
            }
            else {
                maximum_mipmap_count
            };
            b.mipmaps += final_mipmap_count;

            // Resize the bitmap data to fit everything.
            let mut total_pixels = 0;
            iterate_base_map_and_mipmaps(b.width, b.height, 1, 1, final_mipmap_count, |m| {
                total_pixels += m.size;
            });
            b.pixels_float.resize(total_pixels, ColorARGB::default());

            // Now generate mipmaps.
            iterate_base_map_and_mipmaps(b.width, b.height, 1, 1, final_mipmap_count, |m| {
                let map_pixel_count = m.size;

                // Get this bitmap's pixels and the next mipmap's.
                let (map_pixels, next_map_pixels) = &mut b.pixels_float[m.pixel_offset..].split_at_mut(map_pixel_count);

                // Now generate the next bitmap's mipmaps.
                if m.index < final_mipmap_count {
                    let next_map_width = (m.width / 2).max(1);
                    let next_map_height = (m.height / 2).max(1);
                    let next_map_pixel_count = next_map_width * next_map_height;
                    let next_map_pixels = &mut next_map_pixels[..next_map_pixel_count];

                    for y in 0..next_map_height {
                        for x in 0..next_map_width {
                            let mut total_color = ColorARGB::default();
                            let mut count = 0usize;

                            let x_orig = x*2;
                            let y_orig = y*2;

                            for y_prev in y_orig..(y_orig+2).min(m.height) {
                                for x_prev in x_orig..(x_orig+2).min(m.width) {
                                    let copied_color = &map_pixels[x_prev + y_prev * m.width];

                                    // square it to account for sRGB to prevent darkening of gradients
                                    if gamma_corrected_mipmaps {
                                        total_color.a += copied_color.a.powi(2);
                                        total_color.r += copied_color.r.powi(2);
                                        total_color.g += copied_color.g.powi(2);
                                        total_color.b += copied_color.b.powi(2);
                                    }

                                    // otherwise linearly do it (fast but produces worse mipmaps)
                                    else {
                                        total_color.a += copied_color.a;
                                        total_color.r += copied_color.r;
                                        total_color.g += copied_color.g;
                                        total_color.b += copied_color.b;
                                    }

                                    count += 1;
                                }
                            }

                            // Divide to get the average
                            total_color.a /= count as f32;
                            total_color.r /= count as f32;
                            total_color.g /= count as f32;
                            total_color.b /= count as f32;

                            // Then square-root if we were using gamma correction
                            if gamma_corrected_mipmaps {
                                total_color.a = total_color.a.sqrt();
                                total_color.r = total_color.r.sqrt();
                                total_color.g = total_color.g.sqrt();
                                total_color.b = total_color.b.sqrt();
                            }

                            next_map_pixels[x + y * next_map_width] = total_color;
                        }
                    }

                    // Nearest neighbor alpha?
                    if nearest_neighbor_alpha_mipmap {
                        for y in 0..next_map_height {
                            for x in 0..next_map_width {
                                next_map_pixels[x + y * next_map_width].a = map_pixels[x*2 + y*2 * m.width].a;
                            }
                        }
                    }
                }
            });
        }
    }
}

