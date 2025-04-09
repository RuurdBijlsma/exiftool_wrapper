//! Concise example: Read single tags (JSON, String, Option<String>).
use exiftool::{ExifTool, ExifToolError};
use std::path::Path;

const IMAGE_PATH: &str = "data/IMG_20170801_162043.jpg";

fn main() -> Result<(), ExifToolError> {
    let mut et = ExifTool::new()?;
    let path = Path::new(IMAGE_PATH);

    // 1. Read as raw JSON Value
    let make_json = et.json_tag(path, "Make")?;
    println!("Make (JSON): {}", make_json); // Output: "Huawei"

    // 2. Read and deserialize into String
    let model: String = et.read_tag(path, "Model")?;
    println!("Model (String): {}", model); // Output: Nexus 6P

    // 3. Read existing tag into Option<T>
    let width_opt: Option<u32> = et.read_tag(path, "ImageWidth")?;
    println!("Width (Option<u32>): {:?}", width_opt); // Output: Some(2688)

    // 4. Read non-existent tag into Option<T> -> Ok(None)
    let comment_opt: Option<String> = et.read_tag(path, "UserComment")?;
    println!("UserComment (Option<String>): {:?}", comment_opt); // Output: None

    // 5. Attempting to read non-existent tag into String -> Err(TagNotFound)
    match et.read_tag::<String>(path, "NonExistentTag") {
        Err(ExifToolError::TagNotFound { tag, .. }) => {
            println!(
                "Correctly failed to read required tag '{}': TagNotFound",
                tag
            );
        }
        _ => panic!("Expected TagNotFound!"),
    }

    Ok(())
}
