use super::*;

pub(crate) struct SpriteProcessor {
    warnings: Vec<ErrorMessage>
}

impl SpriteProcessor {
    pub fn process_sprites(color_plate: &mut ColorPlate, warnings: &mut Vec<ErrorMessage>) -> ErrorMessageResult<()> {
        if !color_plate.options.bake_sprite_sheets {
            return Ok(())
        }

        // Get our parameters
        let spacing = color_plate.options.preferred_sprite_spacing;

        // Determine budgeting
        let (max_sheet_count, max_sheet_length) = match color_plate.options.sprite_budget_count {
            Some(n) => (n, color_plate.options.sprite_budget_length),
            None => {
                let mut max_sheet_length = 1024;
                let max_sheet_count = 32767;

                for b in &color_plate.bitmaps {
                    max_sheet_length = max_sheet_length.max(b.width).max(b.height)
                }

                (max_sheet_count, max_sheet_length)
            }
        };

        // Pick sprites out of everything.
        let mut sprites = Vec::with_capacity(color_plate.bitmaps.len());
        for s in 0..color_plate.sequences.len() {
            let sequence = &color_plate.sequences[s];
            let first_bitmap = match sequence.first_bitmap {
                Some(n) => n,
                None => continue
            };
            for b in first_bitmap..first_bitmap + sequence.bitmap_count {
                let bitmap = &color_plate.bitmaps[b];
                sprites.push(UnbakedSprite {
                    original_bitmap_index: b,
                    width: bitmap.width,
                    height: bitmap.height,
                    position: Point2DUInt::default()
                });
            }
        }

        let mut processor = SpriteProcessor { warnings: Vec::new() };
        let mut sheets = processor.generate_sheets(max_sheet_length, spacing, color_plate)?;

        let mut total_pixel_usage = 0;
        let max_pixel_usage = max_sheet_length * max_sheet_length * max_sheet_count;

        // Optimize sheets. Then calculate total pixel usage
        for i in &mut sheets {
            i.optimize(color_plate.options.force_square_sheets);
            total_pixel_usage += i.max_length * i.max_height.unwrap_or(i.max_length);
        }

        // Failure?
        if total_pixel_usage > max_pixel_usage {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.bitmap.error_sprite_max_budget_exceeded"), total=total_pixel_usage, max=max_pixel_usage)));
        }

        // Add new bitmaps
        let mut new_bitmaps = Vec::with_capacity(sheets.len());
        let mut sprite_indices = Vec::with_capacity(sheets.len());

        for sheet in &sheets {
            new_bitmaps.push(ColorPlateBitmap {
                pixels: sheet.bake_sprite_sheet(color_plate),
                width: sheet.max_length,
                height: sheet.max_height.unwrap_or(sheet.max_length),
                registration_point: Point2D::default()
            });

            let mut v = Vec::with_capacity(sheet.sprites.len());
            for sprite in &sheet.sprites {
                v.push(sprite.original_bitmap_index);
            }
            sprite_indices.push(v);
        }

        for s in &mut color_plate.sequences {
            let b = match s.first_bitmap {
                Some(n) => n,
                None => continue
            };

            // Find all bitmaps and add them
            'bitmap_loop: for bitmap_index in b..b+s.bitmap_count {
                for sheet_index in 0..sheets.len() {
                    let sheet = &sheets[sheet_index];
                    for sprite in &sheet.sprites {
                        if sprite.original_bitmap_index == bitmap_index {
                            let original_bitmap_index = sprite.original_bitmap_index;
                            let sequence_first_bitmap = s.first_bitmap.unwrap();

                            if !(sequence_first_bitmap..sequence_first_bitmap + s.bitmap_count).contains(&original_bitmap_index) {
                                continue
                            }

                            let original_bitmap = &color_plate.bitmaps[original_bitmap_index];
                            let original_registration_point = original_bitmap.registration_point;

                            s.sprites.push(Sprite {
                                bitmap_index: sheet_index,
                                position: sprite.position,
                                width: sprite.effective_width(&sheet),
                                height: sprite.effective_height(&sheet),
                                registration_point: Point2D {
                                    x: ((original_registration_point.x * (original_bitmap.width as f32)) + (sheet.spacing as f32)) / (sheet.max_length as f32),
                                    y: ((original_registration_point.y * (original_bitmap.height as f32)) + (sheet.spacing as f32)) / (sheet.max_height.unwrap_or(sheet.max_length) as f32),
                                }
                            });

                            continue 'bitmap_loop;
                        }
                    }
                }
            }

            // Just set it to the first sprite's bitmap.
            //
            // It may or may not be accurate to the whole sequence if it is split across multiple sheets, but whatever.
            s.first_bitmap = Some(s.sprites[0].bitmap_index);
            s.bitmap_count = 0;
        }
        color_plate.bitmaps = new_bitmaps;

        warnings.append(&mut processor.warnings);

        Ok(())
    }

    fn generate_sheets(&mut self, max_length: usize, spacing: usize, color_plate: &mut ColorPlate) -> ErrorMessageResult<Vec<UnbakedSpriteSheet>> {
        let mut sprite_sheets = Vec::new();

        // Sort sprites by height in descending order
        let sequence_count = color_plate.sequences.len();

        // Although, if we don't have any sequences, we're done
        if sequence_count == 0 {
            return Ok(sprite_sheets);
        }

        // Otherwise let's continue
        let mut sorted_sprites = Vec::new();
        sorted_sprites.resize(sequence_count, Vec::<usize>::new());

        // Sort sprites from largest to smallest
        for si in 0..sequence_count {
            let seq = &color_plate.sequences[si];
            if seq.first_bitmap.is_none() {
                continue
            }

            let first_bitmap = seq.first_bitmap.unwrap();
            let sorted = &mut sorted_sprites[si];
            *sorted = (first_bitmap..first_bitmap+seq.bitmap_count).collect();

            // Sort by height
            sorted.sort_by(|a,b| color_plate.bitmaps[*b].height.cmp(&color_plate.bitmaps[*a].height));
        }

        // Number of split across sprite sequences (hopefully zero but entirely possible)
        let mut split_across = 0usize;

        // Sort sequences by height.
        //
        // Death Island, free-for-all Slayer. Winner gets the girl.
        let mut sequences_sorted: Vec<usize> = (0..sequence_count).collect();
        sequences_sorted.sort_by(|a,b| {
            let a_seq = &sorted_sprites[*a];
            let b_seq = &sorted_sprites[*b];

            let a_height = match a_seq.first() {
                Some(n) => color_plate.bitmaps[*n].height,
                None => 0
            };

            let b_height = match b_seq.first() {
                Some(n) => color_plate.bitmaps[*n].height,
                None => 0
            };

            b_height.cmp(&a_height)
        });

        // Place them now
        'seq_iter: for si in sequences_sorted {
            // Get our indices
            let sorted = &sorted_sprites[si];

            // Try placing in existing sprite sheets
            for s in &mut sprite_sheets {
                if s.add_sequence_to_sheet(sorted, si, color_plate) {
                    continue 'seq_iter;
                }
            }

            // Make a new sprite sheet if we have to
            let mut new_sprite_sheet = UnbakedSpriteSheet::new(spacing, max_length);

            // Try placing in the new sprite sheet
            if new_sprite_sheet.add_sequence_to_sheet(sorted, si, color_plate) {
                sprite_sheets.push(new_sprite_sheet);
                continue 'seq_iter;
            }

            // If we can't put it in a new sprite sheet, try splitting across the sprite sheets
            else {
                split_across += 1;

                let new_offset = sprite_sheets.len();
                sprite_sheets.push(UnbakedSpriteSheet::new(spacing, max_length));
                let mut next_sheet = &mut sprite_sheets[new_offset];

                // Go through each sprite
                for sprite in sorted {
                    // Attempt to add it to this sheet
                    if !next_sheet.add_sprite_to_sheet(*sprite, &color_plate, true) {
                        // If we fail, move onto the next sheet
                        let new_offset = sprite_sheets.len();
                        sprite_sheets.push(UnbakedSpriteSheet::new(spacing, max_length));
                        next_sheet = &mut sprite_sheets[new_offset];

                        // If we can't even fit it in a sheet by itself, then get rekt
                        if !next_sheet.add_sprite_to_sheet(*sprite, &color_plate, true) {
                            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.bitmap.error_sprite_sheet_cannot_fit"), sequence=si, length=max_length)));
                        }
                    }
                }
            }
        }

        // If we split it across multiple sheets, complain but continue
        if split_across > 0 {
            self.warnings.push(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.bitmap.warning_sequence_split_across_textures"), split_across=split_across)));
        }

        // Done
        Ok(sprite_sheets)
    }
}

/// Information for an unprocessed sprite sheet to bake.
struct UnbakedSpriteSheet {
    /// padding (only meaningful if >1 sprite in this sheet)
    spacing: usize,

    /// max length
    max_length: usize,

    /// max height? (note that non-square sprite sheets will break particles)
    max_height: Option<usize>,

    /// The sprite sheet is locked (no more sprites can be added)
    locked: bool,

    /// All sprites
    sprites: Vec<UnbakedSprite>
}

impl UnbakedSpriteSheet {
    fn new(spacing: usize, max_length: usize) -> UnbakedSpriteSheet {
        debug_assert!(spacing > 0, "can't initialize a sprite sheet with null spacing");
        UnbakedSpriteSheet {
            spacing, max_length, max_height: None, locked: false, sprites: Vec::new()
        }
    }

    fn optimize(&mut self, force_square: bool) {
        // If we only have 1 sprite, remove spacing
        // This is very VERY horrible, inconsistent, and arbitrary as fuck (and not trivial to account for when making your sprites) but it's required to match Bungie output
        if self.sprites.len() == 1 {
            self.spacing = 0;
        }

        // Get the max length needed for our current set of sprites
        let mut max_length_needed = 0usize;
        for s in &self.sprites {
            max_length_needed = (s.position.x as usize + s.effective_width(self)).max(max_length_needed);
            max_length_needed = (s.position.y as usize + s.effective_height(self)).max(max_length_needed);
        }

        // Find the closest power of two (rounded up)
        self.max_length = 1;
        while self.max_length < max_length_needed {
            self.max_length *= 2;
        }

        // If we have more than 1 sprite, brute force a smaller sprite sheet
        if self.sprites.len() > 1 && !self.locked {
            'length_loop: while self.max_length > 1 {
                // Copy the old values
                let old_max_length = self.max_length;
                let old_sprites = self.sprites.clone();

                // Halve max length, clear sprites
                self.max_length /= 2;
                self.sprites.clear();

                // Go through each sprite and see if we can re-add all of them again
                for s in &old_sprites {
                    // Copy the sprite
                    let mut s = s.to_owned();

                    // Fail - copy back in old values
                    if !s.place_in_sheet(self) {
                        self.max_length = old_max_length;
                        self.sprites = old_sprites;
                        break 'length_loop;
                    }

                    // Success - added!
                    self.sprites.push(s);
                }
            }
        }

        // Bad
        if !force_square {
            let mut new_max_height = 0usize;
            for s in &self.sprites {
                new_max_height = (s.position.y as usize + s.effective_height(self)).max(new_max_height);
            }

            let mut max_height = 1;
            while max_height < new_max_height {
                max_height *= 2;
            }
            self.max_height = Some(max_height);
        }
    }

    fn add_sequence_to_sheet(&mut self, sprite_indices: &[usize], sequence: usize, color_plate: &ColorPlate) -> bool {
        // Locked? No then.
        if self.locked {
            return false;
        }

        let seq = &color_plate.sequences[sequence];
        let first_bitmap = match seq.first_bitmap {
            Some(n) => n,
            None => return true // here's your sequence (motions hand to give you something, but the hand is empty) have fun~
        };

        let new_sheet = self.sprites.is_empty();

        // Are we making a new, empty sheet?
        if new_sheet {
            // If we're only adding 1 sprite and we have no sprites, we can handle it a little different
            if sprite_indices.len() == 1 {
                self.add_sprite_to_sheet(first_bitmap, color_plate, true)
            }

            // Try adding everything.
            else {
                let sprite_data_backup = self.sprites.clone();
                for sprite in sprite_indices {
                    if !self.add_sprite_to_sheet(*sprite, color_plate, false) {
                        self.sprites = sprite_data_backup;
                        return false;
                    }
                }

                // Yay!
                true
            }
        }

        // Otherwise, we'll need to sort everything by height and attempt to add everything that way
        else {
            // Bitmap indices
            let mut sorted = Vec::with_capacity(self.sprites.len());

            // Add all sprites already in the sheet as-is since they're already sorted
            for l in &self.sprites {
                sorted.push(l.original_bitmap_index);
            }

            // Get the height of the sprite at sequence
            let sprite_height = |sprite: usize| -> usize {
                color_plate.bitmaps[sprite].height
            };

            // Sort the sprites of the new sequence into these sprites using binary searching to maintain sorting order
            for sprite in sprite_indices {
                let height = sprite_height(*sprite);

                let new_index = match sorted.binary_search_by(|index| height.cmp(&sprite_height(*index))) {
                    Ok(n) => n,
                    Err(n) => n
                };

                sorted.insert(new_index, *sprite);
            }

            // Let's try adding everything
            let sprite_data_backup = self.sprites.clone();
            self.sprites.clear();

            for sprite in sorted {
                if !self.add_sprite_to_sheet(sprite, color_plate, false) {
                    self.sprites = sprite_data_backup;
                    return false;
                }
            }

            // We did it
            true
        }
    }

    /// Calculate the best place to add a sprite to a sheet and return the offset (if possible)
    fn best_place_to_add_sprite(&self, bitmap: usize, color_plate: &ColorPlate) -> Option<UnbakedSprite> {
        // Instantiate this
        let b = &color_plate.bitmaps[bitmap];
        let mut sprite_candidate = UnbakedSprite {
            original_bitmap_index: bitmap,
            width: b.width,
            height: b.height,
            position: Point2DUInt::default()
        };

        // Attempt to place it in the sheet
        if sprite_candidate.place_in_sheet(self) {
            Some(sprite_candidate)
        }
        else {
            None
        }
    }

    /// Add the sprite to the sprite sheet.
    ///
    /// If `allow_removing_spacing` is set to `true`, then spacing can be set to 0 to add the sprite by itself if there are no sprites in the sprite sheet.
    fn add_sprite_to_sheet(&mut self, bitmap: usize, color_plate: &ColorPlate, allow_removing_spacing: bool) -> bool {
        // If locked, don't add anymore sprites
        if self.locked {
            return false;
        }

        // Attempt to add it
        match self.best_place_to_add_sprite(bitmap, color_plate) {
            Some(sprite) => {
                self.sprites.push(sprite);
                return true;
            },
            _ => ()
        }

        // Check if we can add it without spacing
        if allow_removing_spacing && self.sprites.is_empty() {
            let old_spacing = self.spacing;
            self.spacing = 0;
            let added = self.add_sprite_to_sheet(bitmap, color_plate, false);
            if added {
                self.locked = true; // lock the sprite sheet, preventing further sprites from being added
                return added;
            }
            self.spacing = old_spacing;
        }

        false
    }

    /// Bake the sprite sheet into a bitmap.
    fn bake_sprite_sheet(&self, color_plate: &ColorPlate) -> Vec<ColorARGBInt> {
        let sprite_usage = color_plate.options.sprite_sheet_usage;
        let background_color = sprite_usage.get_background_color();
        let mut image = Vec::new();
        image.resize(self.max_length * self.max_height.unwrap_or(self.max_length), background_color);

        // If we're doing multiply, do alpha blend. Otherwise do a simple replace.
        let blend_function = match sprite_usage {
            // alpha blend
            SpriteUsage::MultiplyMin => |bg: &mut ColorARGBInt, fg: ColorARGBInt| {
                let bg_float: ColorARGB = (*bg).into();
                let fg_float: ColorARGB = fg.into();
                *bg = bg_float.alpha_blend(fg_float).into()
            },
            _ => |bg: &mut ColorARGBInt, fg: ColorARGBInt| *bg = fg,

        };

        // Go through each sprite
        for sprite in &self.sprites {
            let bitmap_data = &color_plate.bitmaps[sprite.original_bitmap_index];
            let pixel_data = &bitmap_data.pixels;
            let width = bitmap_data.width;
            let height = bitmap_data.height;

            for y in 0..height {
                for x in 0..width {
                    let pixel_x = x + sprite.position.x as usize + self.spacing;
                    let pixel_y = y + sprite.position.y as usize + self.spacing;
                    blend_function(&mut image[pixel_x + pixel_y * self.max_length], pixel_data[x + y * width]);
                }
            }
        }
        image
    }
}

/// Information for an unprocessed sprite.
#[derive(Copy, Clone, Debug)]
struct UnbakedSprite {
    /// Index of the bitmap.
    original_bitmap_index: usize,

    /// Width of the sprite bitmap.
    width: usize,

    /// Height of the sprite bitmap.
    height: usize,

    /// Position of the sprite
    position: Point2DUInt
}

impl UnbakedSprite {
    fn effective_width(&self, sheet: &UnbakedSpriteSheet) -> usize {
        self.width + sheet.spacing * 2
    }

    fn effective_height(&self, sheet: &UnbakedSpriteSheet) -> usize {
        self.height + sheet.spacing * 2
    }

    /// Attempt to place the sprite in the sheet
    fn place_in_sheet(&mut self, sheet: &UnbakedSpriteSheet) -> bool {
        let max_length = sheet.max_length;

        let width = self.effective_width(sheet);
        let height = self.effective_height(sheet);

        // If the sprite is too big, fail
        if width > max_length || height > max_length {
            return false;
        }

        let old_position = self.position;
        self.position = Point2DUInt::default();

        let x_end = max_length - width;
        let y_end = max_length - height;

        let (alignment, alignment_inv) = match sheet.spacing {
            n if n < 4 => (1, !0),
            _ => (4, !4)
        };

        // 0 sprites always succeeds
        if sheet.sprites.is_empty() {
            return true;
        }

        let mut search_row = |y: usize| {
            self.position.y = y as u16;

            // Scan left to right
            for x in (0..=x_end).step_by(alignment) {
                let mut overlap_fail = false;
                self.position.x = x as u16;
                for s in &sheet.sprites {
                    if s.overlaps(self, sheet) {
                        overlap_fail = true;
                        break;
                    }
                }

                // Success
                if !overlap_fail {
                    return true;
                }
            }

            // Fail
            false
        };

        let mut sprites_level = 0;
        let mut y = 0;

        // Try packing it nicely
        'search_loop: while y <= y_end {
            // Scan left to right
            if search_row(y) {
                return true;
            }

            // Find the next y "level"
            //
            // Essentially we pack in rows. This isn't the most efficient algorithm in the world, but it's fast.
            for l in sprites_level..sheet.sprites.len() {
                let sprite = &sheet.sprites[l];
                if sprite.position.y as usize == y {
                    y += sprite.effective_height(sheet);
                    y += (alignment - (y % alignment)) & alignment_inv; // align to alignment

                    sprites_level = l;
                    continue 'search_loop;
                }
            }
        }

        self.position = old_position;
        false
    }

    fn overlaps(&self, other: &UnbakedSprite, sheet: &UnbakedSpriteSheet) -> bool {
        let overlap = |a: &UnbakedSprite, b: &UnbakedSprite| -> bool {
            let ax = a.position.x as usize;
            let ay = a.position.y as usize;
            let bx = b.position.x as usize;
            let by = b.position.y as usize;

            // Get edges
            let a_end_x = ax + a.effective_width(sheet);
            let a_end_y = ay + a.effective_height(sheet);
            let b_end_x = bx + b.effective_width(sheet);
            let b_end_y = by + b.effective_height(sheet);

            // Left and top
            let l_border_inside = ax >= bx && ax < b_end_x;
            let t_border_inside = ay >= by && ay < b_end_y;

            // Right and bottom
            let r_border_inside = a_end_x > bx && a_end_x < b_end_x; // the "end" is actually the pixel after the last pixel and not technically inside
            let b_border_inside = a_end_y > by && a_end_y < b_end_y;

            // If both a horizontal and vertical border are inside, then it's overlapping
            if (l_border_inside || r_border_inside) && (t_border_inside || b_border_inside) {
                return true;
            }

            // Get borders that are outside
            let l_border_left = ax <= bx;
            let r_border_right = a_end_x >= b_end_x;
            let t_border_up = ay <= by;
            let b_border_bottom = a_end_y >= b_end_y;

            // If either a horizontal or vertical border is inside and we're within the two other borders, then it's overlapping
            if (l_border_inside || r_border_inside) && (t_border_up && b_border_bottom) {
                return true;
            }
            if (t_border_inside || b_border_inside) && (l_border_left && r_border_right) {
                return true;
            }

            // If we're within all borders, we're overlapping
            if l_border_left && r_border_right && t_border_up && b_border_bottom {
                return true;
            }

            false
        };

        overlap(self, other) || overlap(other, self)
    }
}
