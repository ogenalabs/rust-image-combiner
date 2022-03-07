mod args;
use args::Args;
use image::{
    imageops::FilterType::Nearest, io::Reader, DynamicImage, GenericImageView, ImageError,
    ImageFormat,
};
use std::convert::TryInto;

#[derive(Debug)]
enum ImageDataErrors {
    DifferentImageFormats,
    BufferTooSmall,
    UnableToReadImageFromPath(std::io::Error),
    UnableToFormatImage(String),
    UnableToDecodeImage(ImageError),
    UnableToSaveImage(ImageError),
}

struct FloatingImage {
    width: u32,
    height: u32,
    data: Vec<u8>,
    name: String,
}

impl FloatingImage {
    fn new(width: u32, height: u32, name: String) -> Self {
        let buffer_size = height * width * 4;
        let buffer: Vec<u8> = Vec::with_capacity(buffer_size.try_into().unwrap());
        FloatingImage {
            width,
            height,
            data: buffer,
            name,
        }
    }
    fn set_data(&mut self, data: Vec<u8>) -> Result<(), ImageDataErrors> {
        if data.len() > self.data.capacity() {
            return Err(ImageDataErrors::BufferTooSmall);
        }
        self.data = data;
        Ok(())
    }
}

fn main() -> Result<(), ImageDataErrors> {
    let args = Args::new();
    let (image_1, image_format_1) = get_image_from_path(args.image_1)?;
    let (image_2, image_format_2) = get_image_from_path(args.image_2)?;

    if image_format_1 != image_format_2 {
        return Err(ImageDataErrors::DifferentImageFormats);
    }

    let (image_1, image_2) = standardize_size(image_1, image_2);
    let mut output = FloatingImage::new(image_1.width(), image_1.height(), args.output);
    let combined_data = combine_images(image_1, image_2);
    output.set_data(combined_data)?;

    if let Err(e) = image::save_buffer_with_format(
        output.name,
        &output.data,
        output.width,
        output.height,
        image::ColorType::Rgba8,
        image_format_1,
    ) {
        Err(ImageDataErrors::UnableToSaveImage(e))
    } else {
        Ok(())
    }
}

fn get_image_from_path(path: String) -> Result<(DynamicImage, ImageFormat), ImageDataErrors> {
    match Reader::open(&path) {
        Ok(reader) => {
            if let Some(format) = reader.format() {
                match reader.decode() {
                    Ok(image) => Ok((image, format)),
                    Err(e) => Err(ImageDataErrors::UnableToDecodeImage(e)),
                }
            } else {
                return Err(ImageDataErrors::UnableToFormatImage(path));
            }
        }
        Err(e) => Err(ImageDataErrors::UnableToReadImageFromPath(e)),
    }
}

fn get_smallest_dimensions(dim_1: (u32, u32), dim_2: (u32, u32)) -> (u32, u32) {
    let pix_1 = dim_1.0 * dim_1.1;
    let pix_2 = dim_2.0 * dim_2.1;

    return if pix_1 < pix_2 { dim_1 } else { dim_2 };
}

fn standardize_size(image_1: DynamicImage, image_2: DynamicImage) -> (DynamicImage, DynamicImage) {
    let (width, height) = get_smallest_dimensions(image_1.dimensions(), image_2.dimensions());
    let image_1 = image_1.resize_exact(width, height, Nearest);
    let image_2 = image_2.resize_exact(width, height, Nearest);

    return (image_1, image_2);
}

fn combine_images(image_1: DynamicImage, image_2: DynamicImage) -> Vec<u8> {
    let vec_1 = image_1.to_rgba8().into_vec();
    let vec_2 = image_2.to_rgba8().into_vec();

    return alternate_pixels(vec_1, vec_2);
}

fn alternate_pixels(vec_1: Vec<u8>, vec_2: Vec<u8>) -> Vec<u8> {
    let mut vec_out: Vec<u8> = Vec::new();

    for i in 0..vec_1.len() {
        if i % 4 == 0 {
            vec_out.push(vec_1[i]);
        } else {
            vec_out.push(vec_2[i]);
        }
    }

    return vec_out;
}
