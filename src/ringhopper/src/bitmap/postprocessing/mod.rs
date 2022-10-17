use crate::types::{ColorARGBInt, log2_u16, ColorARGB, Vector3D, Point2D};

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
    pub faces: usize
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

/// Options for configuring the bitmap processor.
#[derive(Copy, Clone, Default)]
pub struct ProcessingOptions {
    /// If `Some`, convert a monochrome input into a bumpmap, setting the height to make the bumpmap.
    pub bumpmap_height: Option<f64>,

    /// Algorithm to use for bumpmaps.
    pub bumpmap_algorithm: BumpmapAlgorithm,

    /// If `Some`, fade mipmaps to gray by a factor when doing mipmap generation.
    pub detail_fade_factor: Option<f64>,

    /// If `Some`, sharpen the base map and mipmaps by a factor when doing mipmap generation.
    pub sharpen_factor: Option<f64>,

    /// If `Some`, blur the base map and mipmaps by a factor when doing mipmap generation.
    pub blur_factor: Option<f64>,

    /// If `Some`, modify the alpha by this factor when doing mipmap generation.
    pub alpha_bias: Option<f64>,

    /// If `Some`, limit the maximum number of mipmaps to generate.
    pub max_mipmaps: Option<usize>,

    /// If `true`, truncate pixels with no alpha.
    pub truncate_zero_alpha: bool,

    /// If `true`, vectorize each pixel of the final result.
    pub vectorize: bool,

    /// If `true`, use nearest neighbor scaling for the alpha channel.
    pub nearest_neighbor_alpha_mipmap: bool
}

/// Final processed bitmap sequence.
#[derive(Copy, Clone, Default)]
pub struct ProcessedSequence {
    /// First bitmap in the sequence, if set.
    pub first_bitmap: Option<usize>,

    /// Number of bitmaps in the sequence.
    pub bitmap_count: usize
}

/// Bitmap postprocessor which can generate mipmaps and do postprocessing.
#[derive(Clone)]
pub struct ProcessedBitmaps {
    /// All final sequences.
    pub sequences: Vec<ProcessedSequence>,

    /// All processed bitmaps.
    pub bitmaps: Vec<ProcessedBitmap>
}

impl ProcessedBitmaps {
    /// Process the color plate, returning a final set of bitmaps.
    pub fn process_color_plate(color_plate: ColorPlate, options: &ProcessingOptions) -> ProcessedBitmaps {
        // Load the processed bitmaps.
        let color_plate_type = color_plate.input_type;
        let mut processed_bitmaps = ProcessedBitmaps::load_color_plate(color_plate);

        processed_bitmaps.perform_blur(options);
        processed_bitmaps.generate_heightmaps(options);
        processed_bitmaps.generate_mipmaps(options);
        processed_bitmaps.truncate_zero_alpha(options);
        processed_bitmaps.consolidate_textures(color_plate_type);

        // Vectorize the result?
        if options.vectorize {
            processed_bitmaps.vectorize();
        }

        processed_bitmaps
    }

    /// Use the color plate data to make a set of processed bitmaps.
    fn load_color_plate(color_plate: ColorPlate) -> ProcessedBitmaps {
        let mut processed_bitmaps = ProcessedBitmaps {
            sequences: Vec::new(),
            bitmaps: Vec::new()
        };

        processed_bitmaps.bitmaps.reserve_exact(color_plate.bitmaps.len());
        processed_bitmaps.sequences.reserve_exact(color_plate.sequences.len());

        // Copy in the bitmaps.
        for b in color_plate.bitmaps {
            processed_bitmaps.bitmaps.push(ProcessedBitmap { pixels: b.pixels.clone(), height: b.height, width: b.width, depth: 1, mipmaps: 0, faces: 1 })
        }

        // Copy in the sequences.
        for s in color_plate.sequences {
            processed_bitmaps.sequences.push(ProcessedSequence { first_bitmap: s.first_bitmap, bitmap_count: s.bitmap_count })
        }

        processed_bitmaps
    }

    /// If a bitmap has zero alpha, set to black.
    fn truncate_zero_alpha(&mut self, options: &ProcessingOptions) {
        const BLACK: ColorARGBInt = ColorARGBInt { a: 0, r: 0, g: 0, b: 0 };

        if !options.truncate_zero_alpha {
            return
        }

        for b in &mut self.bitmaps {
            for p in &mut b.pixels {
                if p.a == 0 {
                    *p = BLACK;
                }
            }
        }
    }

    /// Perform a blur
    fn perform_blur(&mut self, options: &ProcessingOptions) {
        // Apply sharpening and blur?
        if let Some(blur) = options.blur_factor {
            for b in &mut self.bitmaps {
                let mut output = Vec::<ColorARGBInt>::with_capacity(b.pixels.len());

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

                                let real_x2 = (x as isize - x2_delta).clamp(0, b.width as isize - 1) as usize;
                                let real_y2 = (y as isize - y2_delta).clamp(0, b.height as isize - 1) as usize;

                                let point_c = Point2D { x: real_x2 as f32, y: real_y2 as f32 };
                                let distance_squared = point_c.distance_from_point_squared(&point);
                                if distance_squared >= max_distance_squared {
                                    continue;
                                }

                                let color: ColorARGB = b.pixels[real_x2 + real_y2 * b.width].into();
                                let factor = 1.0 - (distance_squared / max_distance_squared).sqrt();

                                total_factor += factor;
                                total_color.a += color.a.powi(2) * factor; // square it to account for sRGB
                                total_color.r += color.r.powi(2) * factor;
                                total_color.g += color.g.powi(2) * factor;
                                total_color.b += color.b.powi(2) * factor;
                            }
                        }

                        total_color.a = (total_color.a / total_factor).sqrt();
                        total_color.r = (total_color.r / total_factor).sqrt();
                        total_color.g = (total_color.g / total_factor).sqrt();
                        total_color.b = (total_color.b / total_factor).sqrt();
                        output.push(total_color.into());
                    }
                }

                for i in 0..output.len() {
                    b.pixels[i] = output[i];
                }
            }
        }
    }

    /// Consolidate cubemaps and 3D textures into one texture.
    fn consolidate_textures(&mut self, color_plate_type: ColorPlateInputType) {
        if color_plate_type != ColorPlateInputType::ThreeDimensionalTextures && color_plate_type != ColorPlateInputType::Cubemaps {
            return;
        }

        let mut new_bitmaps = Vec::new();
        let mut new_sequences = Vec::with_capacity(self.sequences.len());

        for s in &self.sequences {
            if s.first_bitmap.is_none() {
                new_sequences.push(*s);
                continue;
            }

            let first_bitmap_index = s.first_bitmap.unwrap();
            let first_bitmap = &self.bitmaps[first_bitmap_index];
            let width = first_bitmap.width;
            let height = first_bitmap.height;
            let (depth, faces) = match color_plate_type {
                ColorPlateInputType::Cubemaps => (1, s.bitmap_count),
                ColorPlateInputType::ThreeDimensionalTextures => (s.bitmap_count, 1),
                _ => unreachable!()
            };
            let mipmaps = first_bitmap.mipmaps;

            let mut capacity = 0;
            for b in first_bitmap_index..first_bitmap_index+s.bitmap_count {
                capacity += self.bitmaps[b].pixels.len();
            }

            let mut new_bitmap = ProcessedBitmap { pixels: Vec::with_capacity(capacity), height, width, depth, mipmaps, faces };
            iterate_base_map_and_mipmaps(width, height, 1, 1, mipmaps, |m| {
                for b in &self.bitmaps[first_bitmap_index..first_bitmap_index+s.bitmap_count] {
                    new_bitmap.pixels.extend_from_slice(&b.pixels[m.pixel_offset..m.pixel_offset+m.size]);
                }
            });

            new_sequences.push(ProcessedSequence { first_bitmap: Some(new_bitmaps.len()), bitmap_count: 1 });
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
            let mut new_pixels = Vec::<ColorARGBInt>::with_capacity(b.pixels.len());
            iterate_base_map_and_mipmaps(b.width, b.height, 1, b.depth, b.mipmaps, |m| {
                // For the first index, just passthrough it.
                let mipmap_size = m.size / b.depth;
                let compaction = 1 << m.index;
                let depth = (b.depth / compaction).max(1);

                for i in 0..depth {
                    let first_level = i * compaction;
                    let end = (i + 1) * compaction;

                    let mut new_pixels_buffer = Vec::<ColorARGBInt>::with_capacity(mipmap_size);

                    for p in 0..mipmap_size {
                        let mut new_pixel = ColorARGB::default();

                        for l in first_level..end {
                            let adding_pixel: ColorARGB = b.pixels[m.pixel_offset + l * mipmap_size + p].into();
                            new_pixel.a += adding_pixel.a;
                            new_pixel.r += adding_pixel.r;
                            new_pixel.g += adding_pixel.g;
                            new_pixel.b += adding_pixel.b;
                        }

                        new_pixel.a /= compaction as f32;
                        new_pixel.r /= compaction as f32;
                        new_pixel.g /= compaction as f32;
                        new_pixel.b /= compaction as f32;

                        new_pixels_buffer.push(new_pixel.into());
                    }

                    new_pixels.append(&mut new_pixels_buffer);
                }
            });

            b.pixels = new_pixels;
        }
    }

    /// Vectorize bitmaps.
    fn vectorize(&mut self) {
        for b in &mut self.bitmaps {
            for p in &mut b.pixels {
                *p = p.vector_normalize();
            }
        }
    }

    /// Generate heightmaps from monochrome input.
    fn generate_heightmaps(&mut self, options: &ProcessingOptions) {
        if options.bumpmap_height.is_none() {
            return;
        }

        let h = options.bumpmap_height.unwrap() as f32;

        for b in &mut self.bitmaps {
            let mut new_pixels = Vec::with_capacity(b.pixels.len());

            for center_y in 0..b.height {
                let top_y = center_y.saturating_sub(1);
                let bottom_y = (center_y + 1).min(b.height - 1);

                for center_x in 0..b.width {
                    let left_x = center_x.saturating_sub(1);
                    let right_x = (center_x + 1).min(b.width - 1);

                    let strength = |x,y| {
                        let pixel: &ColorARGBInt = &b.pixels[x + y * b.width];
                        (pixel.r as f32) / 255.0 * 2.0 - 1.0
                    };

                    let center = strength(center_x, center_y);
                    let top = strength(center_x, top_y);
                    let left = strength(left_x, center_y);
                    let right = strength(right_x, center_y);
                    let bottom = strength(center_x, bottom_y);

                    let (dx, dy, dz);

                    match options.bumpmap_algorithm {
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
                        a: b.pixels[center_x + center_y * b.width].a as f32 / 255.0,
                        r: v.x / 2.0 + 0.5,
                        g: v.y / 2.0 + 0.5,
                        b: v.z / 2.0 + 0.5
                    };

                    new_pixels.push(c.into());
                }
            }

            debug_assert_eq!(b.pixels.len(), new_pixels.len());
            b.pixels = new_pixels;
        }
    }

    /// Generate mipmaps.
    fn generate_mipmaps(&mut self, options: &ProcessingOptions) {
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
            let final_mipmap_count = if let Some(n) = options.max_mipmaps {
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
            b.pixels.resize(total_pixels, ColorARGBInt::default());

            // Now generate mipmaps.
            //
            // Note that, yes, sharpness and blur also apply to the base map.
            iterate_base_map_and_mipmaps(b.width, b.height, 1, 1, final_mipmap_count, |m| {
                let map_pixel_count = m.size;

                // Get this bitmap's pixels and the next mipmap's.
                let (map_pixels, next_map_pixels) = &mut b.pixels[m.pixel_offset..].split_at_mut(map_pixel_count);
                if let Some(sharpen) = options.sharpen_factor {
                    todo!("sharpen not yet implemented {factor}", factor=sharpen)
                }

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
                                    let copied_color: ColorARGB = map_pixels[x_prev + y_prev * m.width].into();

                                    // square it to account for sRGB to prevent darkening of gradients
                                    total_color.a += copied_color.a.powi(2);
                                    total_color.r += copied_color.r.powi(2);
                                    total_color.g += copied_color.g.powi(2);
                                    total_color.b += copied_color.b.powi(2);

                                    count += 1;
                                }
                            }

                            total_color.a /= count as f32;
                            total_color.r /= count as f32;
                            total_color.g /= count as f32;
                            total_color.b /= count as f32;

                            total_color.a = total_color.a.sqrt();
                            total_color.r = total_color.r.sqrt();
                            total_color.g = total_color.g.sqrt();
                            total_color.b = total_color.b.sqrt();

                            next_map_pixels[x + y * next_map_width] = total_color.into();
                        }
                    }

                    // Nearest neighbor alpha?
                    if options.nearest_neighbor_alpha_mipmap {
                        for y in 0..next_map_height {
                            for x in 0..next_map_width {
                                next_map_pixels[x + y * next_map_width].a = map_pixels[x*2 + y*2 * m.width].a;
                            }
                        }
                    }
                }
            });

            // Lastly, fade to gray on mipmaps
            if let Some(fade) = options.detail_fade_factor {
                if final_mipmap_count > 0 {
                    // Note that while OFFICIALLY only mipmaps are mentioned in fade-to-gray by guerilla, calculating fade factor includes the base map too.
                    let fade = fade.clamp(0.0, 1.0) as f32;
                    let mipmap_count_plus_one = final_mipmap_count as f32 + 1.0;
                    let overall_fade_factor = mipmap_count_plus_one - fade * (mipmap_count_plus_one - 1.0 + (1.0 - fade));
                    let pixels = &mut b.pixels[b.width * b.height..];

                    iterate_base_map_and_mipmaps((b.width / 2).max(1), (b.height / 2).max(1), 1, 1, final_mipmap_count - 1, |m| {
                        let a = if fade == 1.0 {
                            // To prevent shenanigans, limit to 1.0
                            1.0
                        }
                        else {
                            // Basically, a higher mipmap fade factor scales faster
                            (((m.index + 1) as f32) / overall_fade_factor).min(1.0)
                        };

                        // Do fade-to-gray on each pixel
                        let fade_to_gray = ColorARGB { a, r: 0.5, g: 0.5, b: 0.5 };
                        for px in &mut pixels[m.pixel_offset..m.pixel_offset + m.size] {
                            let px_float: ColorARGB = (*px).into();
                            *px = px_float.alpha_blend(fade_to_gray).into();
                        }
                    });
                }
            }
        }
    }
}

