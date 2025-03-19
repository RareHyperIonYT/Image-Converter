use std::{error::Error, path::{Path, PathBuf}, io::BufWriter};
use image::{DynamicImage, ImageFormat};

pub fn convert_image(input: &str, output_folder: &str, format: &str) -> Result<(), Box<dyn Error>> {
    let img: DynamicImage = image::open(input)?;
    let input_path = Path::new(input);
    let filename = input_path
        .file_stem()
        .ok_or("Invalid file name")?
        .to_string_lossy();

    let output_path = PathBuf::from(output_folder);
    std::fs::create_dir_all(&output_path)?;

    let output_path = output_path.join(format!("{}.{}", filename, format));
    let file = std::fs::File::create(&output_path)?;
    let mut writer = BufWriter::new(file);

    let output_format = match format {
        "jpeg" | "jpg" => ImageFormat::Jpeg,
        "webp" => ImageFormat::WebP,
        "gif" => ImageFormat::Gif,
        _ => ImageFormat::Png,
    };

    img.write_to(&mut writer, output_format)?;
    Ok(())
}