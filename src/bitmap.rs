use image::RgbaImage;

pub struct BitmapDescriptor {
    img: RgbaImage,
    char_width_pixels: u32,
    char_height_pixels: u32,
    char_gap_x_pixels: u32,
    char_gap_y_pixels: u32,
    uppercase_only: bool,
    lowercase_only: bool,
    pub chars: String, // Used to view available charset
}

pub struct BitmapFont {
    desc: BitmapDescriptor,
    img_width_chars: u32,
    img_height_chars: u32,

    char_jump_tbl: Vec<u32>,
}

pub struct CharUv {
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
}

impl BitmapDescriptor {
    // This guy might be overkill, don't know how rust wants things
    pub fn new(
        img: RgbaImage,
        char_width_pixels: u32, 
        char_height_pixels: u32, 
        char_gap_x_pixels: u32,
        char_gap_y_pixels: u32,
        uppercase_only: bool,
        lowercase_only: bool,
        chars: String) -> Self 
        {
            Self{ 
                img,
                char_width_pixels,
                char_height_pixels,
                char_gap_x_pixels,
                char_gap_y_pixels,
                uppercase_only,
                lowercase_only,
                chars
            }
        }
}

impl BitmapFont {
    pub fn new(desc: BitmapDescriptor) -> Self {
        let img_width_chars = (desc.img.width() + desc.char_gap_x_pixels) /
            (desc.char_width_pixels + desc.char_gap_x_pixels);
        
        let img_height_chars = (desc.img.height() + desc.char_gap_y_pixels) /
            (desc.char_height_pixels + desc.char_gap_y_pixels);
        
        let char_jump_tbl = Self::build_char_jump_tbl(&desc);

        Self { desc, img_width_chars, img_height_chars, char_jump_tbl }
    }

    pub fn image(&self) -> &RgbaImage { &self.desc.img }

    pub fn char_aspect(&self) -> f32 {
        // Aspect ratio
        self.desc.char_width_pixels as f32 / self.desc.char_height_pixels as f32
    }

    fn build_char_jump_tbl(desc: &BitmapDescriptor) -> Vec<u32> {
        // guh syntax
        let mut jump_tbl: Vec<u32> = vec![0; 128];
        let mut count = 0;
        for c in desc.chars.chars() {
            jump_tbl[c as usize] = count;
            count += 1;
        }

        // If both lettercases aren't handled, map the non-handled one to the
        // handled one
        if desc.uppercase_only {
            for c in 'a'..='z' {
                jump_tbl[c as usize] = jump_tbl[c.to_ascii_uppercase() as usize];
            } 
        } else if desc.lowercase_only {
            for c in 'A'..='Z' {
                jump_tbl[c as usize] = jump_tbl[c.to_ascii_lowercase() as usize];
            } 
        }

        return jump_tbl;
    }

    /* 
    fn find_max_char(chars: &String) -> usize {
        let mut max_val = 0;
        for c in chars.chars() {
            if max_val < c as usize {
                max_val = c as usize;
            }
        }
        return max_val;
    }
    */

    pub fn get_char_uv(&self, c: char) -> CharUv {
        // This line should have a fallback bc I want to use this interactively
        // Easiest would be to add '?' to the font and fallback to it
        let index = self.char_jump_tbl[c as usize];

        let x_tiles = index % self.img_width_chars;
        let y_tiles = index / self.img_width_chars;

        let x_pixels = x_tiles * 
            (self.desc.char_width_pixels + self.desc.char_gap_x_pixels);
        let y_pixels = y_tiles * 
            (self.desc.char_height_pixels + self.desc.char_gap_y_pixels);
        
        // Unsure about all the f32 conversions
        // Right now this is just 1:1 with the old C-Version
        let width = self.desc.img.width() as f32;
        let height = self.desc.img.height() as f32;

        CharUv { 
            u0: x_pixels as f32 / width, 
            v0: y_pixels as f32 / height, 
            u1: (x_pixels + self.desc.char_width_pixels) as f32 / width, 
            v1: (y_pixels + self.desc.char_height_pixels) as f32 / height 
        }
    }
}