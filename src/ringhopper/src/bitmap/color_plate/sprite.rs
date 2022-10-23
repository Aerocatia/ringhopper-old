#![allow(dead_code)]

use super::*;

pub(super) fn process_sprites(color_plate: &mut ColorPlate) -> ErrorMessageResult<()> {
    if !color_plate.options.bake_sprite_sheets {
        return Ok(())
    }
    Err(ErrorMessage::StaticString("Sprite sheets are unimplemented!"))
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

/// Information for an unprocessed sprite.
#[derive(Copy, Clone)]
struct UnbakedSprite {
    /// Index of the bitmap.
    original_bitmap_index: usize,

    /// Width of the sprite bitmap.
    width: usize,

    /// Height of the sprite bitmap.
    height: usize,

    /// Index of the sequence.
    sequence: usize,

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
