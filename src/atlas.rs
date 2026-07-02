use image::{ImageBuffer, ImageError, Rgba, imageops};
use serde::{Serialize, Serializer};
use std::collections::HashMap;

use crate::image_extract::TextureData;

// Metadata that describes a texture's placement inside the atlas
#[derive(Serialize)]// We dont de-serialise ever so we dont need it
struct AtlasEntry {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    
    #[serde(
        serialize_with = "bool_as_int"
    )]
    pub rotated: bool,
}

// When the json serialises to write the output json file, it replaces true and false with 1 and 0
pub fn bool_as_int<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u8(if *value { 1 } else { 0 })
}


impl AtlasEntry {
    pub fn new(x: u32, y: u32, width: u32, height: u32, rotated: bool) -> Self {
        return AtlasEntry {
            x,
            y,
            width,
            height,
            rotated,
        };
    }

    pub fn display(&self) {
        print!(
            "x: {}, y: {}, width: {}, height: {}, rotated: {}",
            self.x, self.y, self.width, self.height, self.rotated
        );
    }
}

// Rectangular region used by packing algorithm
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub bx: u32, // Bottom-right x
    pub by: u32, // Bottom-right y
}

impl Region {
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        return Region {
            x,
            y,
            w,
            h,
            bx: x + w,
            by: y + h,
        };
    }

    pub fn from_coords(x1: u32, y1: u32, x2: u32, y2: u32) -> Self {
        return Region {
            x: (x1),
            y: (y1),
            w: (x2 - x1),
            h: (y2 - y1),
            bx: (x2),
            by: (y2),
        };
    }

    // Area can get super inflated so we use a u64 instead of u32
    pub fn area(&self) -> u64 {
        return self.w as u64 * self.h as u64;
    }

    pub fn can_fit(&self, other_w: u32, other_h: u32) -> bool {
        return self.w >= other_w && self.h >= other_h;
    }
}

pub fn gen_atlas(
    mut images: Vec<TextureData>,
    allow_rotation: bool,
) -> Result<(ImageBuffer<Rgba<u8>, Vec<u8>>, String), ImageError> {
    // Sort images by DESCENDING area, so the largest textures are first
    images.sort_by(|b, a| {
        (a.texture.dimensions().0 * a.texture.dimensions().1)
            .cmp(&(b.texture.dimensions().0 * b.texture.dimensions().1))
    });

    let mut atlas_key: HashMap<String, AtlasEntry> = HashMap::new();
    let atlas = pack_atlas(images, &mut atlas_key, allow_rotation);

    // Print info on the generated atlas keys
    // for key in atlas_key.keys() {
    //     print!("\n{} ", key);
    //     atlas_key[key].display();
    // }

    // Serialise the key data
    let json = serde_json::to_string_pretty(&atlas_key).unwrap();

    return Ok((atlas, json));
}

fn pack_atlas(
    mut init_images: Vec<TextureData>,
    atlas_data: &mut HashMap<String, AtlasEntry>,
    allow_rotation: bool,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    // Inital state for atlas dimensions
    let mut lower_bound = 1;
    // Start with an inital guess for the upper bound as the sqrt of every textures width + height added
    let mut upper_bound = init_images
        .iter()
        .fold(0, |acc, x| acc + x.texture.width() + x.texture.height());
    upper_bound = upper_bound.isqrt();

    let mut best_working_size = (upper_bound + lower_bound) / 2;

    // Reused data
    let mut atlas_texture = image::RgbaImage::new(0, 0);
    let mut region_tree: Vec<Region> = vec![];
    let mut image_array: Vec<TextureData> = init_images.to_vec();

    // Keep doubling upper bound utill a valid size is found or failure occurs
    println!("Calibrating upper bound...");
    let mut calibrating = true;
    while calibrating {
        // Reset parameters and attempt packing
        image_array = init_images.clone();
        atlas_data.clear();
        region_tree.clear();
        region_tree.push(Region::new(0, 0, upper_bound, upper_bound));
        atlas_texture = image::RgbaImage::new(upper_bound, upper_bound);
        recursive_pack(
            &mut image_array,
            atlas_data,
            &mut region_tree,
            &mut atlas_texture,
            allow_rotation,
        );

        if image_array.len() == 0 {
            calibrating = false;
        } else {
            upper_bound *= 2;
            println!(
                "Packing failed, images unpacked: {}\nUpper bound increased to: {}",
                image_array.len(),
                upper_bound
            );
        }
    }
    println!(
        "Packing was successful. Upper bound set to: {}",
        upper_bound
    );

    // Binary search on bounding size of atlas
    let mut iteration = 0;
    while (upper_bound - lower_bound) > 1 {
        let mut midpoint = (upper_bound + lower_bound) / 2;

        println!(
            "Iteration: {} [Best size: {}, Lower bound: {}, Upper bound: {}, Current midpoint: {}]",
            iteration, best_working_size, lower_bound, upper_bound, midpoint
        );

        // Reset parameters and attempt packing
        image_array = init_images.clone();
        atlas_data.clear();
        region_tree.clear();
        region_tree.push(Region::new(0, 0, midpoint, midpoint));
        atlas_texture = image::RgbaImage::new(midpoint, midpoint);
        recursive_pack(
            &mut image_array,
            atlas_data,
            &mut region_tree,
            &mut atlas_texture,
            allow_rotation,
        );

        // No images in the array means packing was succesful
        if image_array.len() == 0 {
            // Keep a seperate track of the midpoint whenever packing completes, so we can guarantee a successful pack at the end
            best_working_size = midpoint;
            upper_bound = midpoint;
        } else {
            lower_bound = midpoint;
        }
        iteration += 1;
    }

    // Perform final packing once optimal size has been found
    image_array = init_images.clone();
    atlas_data.clear();
    region_tree.clear();
    region_tree.push(Region::new(0, 0, best_working_size, best_working_size));
    atlas_texture = image::RgbaImage::new(best_working_size, best_working_size);
    recursive_pack(
        &mut image_array,
        atlas_data,
        &mut region_tree,
        &mut atlas_texture,
        allow_rotation,
    );

    return atlas_texture;
}

// After placing an image, there are two possible ways to split the
// remaining free space:
//
// Layout A:
// +------+------
// | IMG  | r1_b |
// +------+------
// |     r1_a    |
// +-------------
//
// Layout B:
// +------+------
// | IMG  |      |
// |      | r2_b |
// +------+      |
// | r2_a |      |
// +------+------
//
// We calculate the area difference between the generated regions and
// choose the split that produces the most balanced remaining space.
fn recursive_pack(
    images: &mut Vec<TextureData>,
    atlas_data: &mut HashMap<String, AtlasEntry>,
    region_tree: &mut Vec<Region>,
    atlas_texture: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    allow_rotation: bool,
) {
    // Get the smallest region
    let this_region = match region_tree.pop() {
        Some(x) => x,
        None => return,
    };

    // find the largest image that will fit inside this region
    for i in 0..images.len() {
        // Check if it fits normally
        // If not, check if it fits rotated,
        // if neither, then fail and move to next image.
        let mut rotated = false;
        if this_region.can_fit(images[i].w, images[i].h) {
            rotated = false;
        }
        // When rotated, h and w are flipped
        else if allow_rotation && this_region.can_fit(images[i].h, images[i].w) {
            rotated = true;
        } else {
            continue;
        }

        let mut img = images.remove(i);

        if rotated {
            img.texture = imageops::rotate90(&img.texture);
            img.rotate();
        }

        //Insert the image at the top-left of this region
        image::imageops::overlay(
            atlas_texture,
            &img.texture,
            this_region.x as i64,
            this_region.y as i64,
        );

        insert_key(atlas_data, &img, this_region.x, this_region.y, rotated);

        // Two sets of 2 subregions, L and S
        // see which set, a or b, has the largest difference between areas and choose the one with the larger difference
        let r1_a = Region::from_coords(
            this_region.x,
            this_region.y + img.h,
            this_region.bx,
            this_region.by,
        );
        let r1_b = Region::new(
            this_region.x + img.w,
            this_region.y,
            this_region.w - img.w,
            img.h,
        );
        let r2_a = Region::new(
            this_region.x,
            this_region.y + img.h,
            img.w,
            this_region.h - img.h,
        );
        let r2_b = Region::from_coords(
            this_region.x + img.w,
            this_region.y,
            this_region.bx,
            this_region.by,
        );

        let a_diff = r1_a.area().abs_diff(r2_a.area());
        let b_diff = r1_b.area().abs_diff(r2_b.area());

        // Push The larger area first, then the smaller area so the smaller area gets processed first
        if a_diff > b_diff {
            if r1_a.area() >= r1_b.area() {
                region_tree.push(r1_a);
                region_tree.push(r1_b);
            } else {
                region_tree.push(r1_b);
                region_tree.push(r1_a);
            }
        } else {
            if r2_a.area() >= r2_b.area() {
                region_tree.push(r2_a);
                region_tree.push(r2_b);
            } else {
                region_tree.push(r2_b);
                region_tree.push(r2_a);
            }
        }

        break;
    }
    recursive_pack(images, atlas_data, region_tree, atlas_texture, allow_rotation);
}

fn insert_key(
    atlas_data: &mut HashMap<String, AtlasEntry>,
    image: &TextureData,
    pos_x: u32,
    pos_y: u32,
    rotated: bool,
) {
    let entry = AtlasEntry::new(
        pos_x,
        pos_y,
        image.texture.dimensions().0,
        image.texture.dimensions().1,
        rotated,
    );
    atlas_data.insert(image.name.clone(), entry);
}
