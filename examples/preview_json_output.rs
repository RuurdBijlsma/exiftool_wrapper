use exiftool_wrapper::exiftool::ExifTool;
use serde_json::{Map, Value};
use std::fs::File;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn list_files_recursive(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok()) // Ignore errors during traversal
        .filter(|e| e.file_type().is_file())
    // Only include files
    {
        files.push(entry.into_path());
    }

    Ok(files)
}

// --- combine_exif_json function remains the same ---
fn combine_exif_json(input_array: &Value) -> Result<Value, String> {
    // Ensure the input is an array
    let input_objects = input_array
        .as_array()
        .ok_or_else(|| "Input value is not an array".to_string())?;

    // This will hold the final combined object
    let mut combined_object: Map<String, Value> = Map::new();

    // Iterate through each object in the input array
    for input_val in input_objects {
        let input_obj = input_val
            .as_object()
            .ok_or_else(|| format!("Input array contains a non-object element: {:?}", input_val))?;

        // Iterate through key-value pairs (Group/Tag -> Value) in the current input object
        for (key, current_value) in input_obj {
            match combined_object.entry(key.clone()) {
                serde_json::map::Entry::Vacant(entry) => {
                    if let Some(current_obj) = current_value.as_object() {
                        let mut nested_map = Map::new();
                        for (nested_key, nested_value) in current_obj {
                            nested_map.insert(
                                nested_key.clone(),
                                Value::Array(vec![nested_value.clone()]),
                            );
                        }
                        entry.insert(Value::Object(nested_map));
                    } else {
                        entry.insert(Value::Array(vec![current_value.clone()]));
                    }
                }
                serde_json::map::Entry::Occupied(mut entry) => {
                    let combined_value = entry.get_mut();
                    if let Some(current_obj) = current_value.as_object() {
                        if let Some(combined_map) = combined_value.as_object_mut() {
                            for (nested_key, nested_value) in current_obj {
                                match combined_map.entry(nested_key.clone()) {
                                    serde_json::map::Entry::Vacant(nested_entry) => {
                                        nested_entry
                                            .insert(Value::Array(vec![nested_value.clone()]));
                                    }
                                    serde_json::map::Entry::Occupied(mut nested_entry) => {
                                        if let Some(arr) = nested_entry.get_mut().as_array_mut() {
                                            if !arr.contains(nested_value) {
                                                arr.push(nested_value.clone());
                                            }
                                        } else {
                                            return Err(format!(
                                                "Type mismatch for key '{}' -> nested key '{}': expected Array, found {:?}",
                                                key, nested_key, nested_entry.get()
                                            ));
                                        }
                                    }
                                }
                            }
                        } else {
                            return Err(format!(
                                "Type mismatch for key '{}': expected Object, found {:?}",
                                key, combined_value
                            ));
                        }
                    } else if let Some(arr) = combined_value.as_array_mut() {
                        if !arr.contains(current_value) {
                            arr.push(current_value.clone());
                        }
                    } else {
                        return Err(format!(
                            "Type mismatch for key '{}': expected Array, found {:?}",
                            key, combined_value
                        ));
                    }
                }
            }
        }
    }

    Ok(Value::Object(combined_object))
}

// Using Result<(), Box<dyn std::error::Error>> for main to easily handle errors
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Hardcoded directory path
    let dir_path = PathBuf::from("test_data/other_images");
    // Output file path
    let output_file_path = "examples/example_output/combined.json";

    // Read directory and collect all regular files
    let files: Vec<PathBuf> = list_files_recursive(&dir_path)?;

    if files.is_empty() {
        println!("No files found in the directory: {}", dir_path.display());
        return Ok(()); // Exit cleanly
    }

    dbg!(&files.len());

    // Convert sampled_files to a Vec<String> for owned paths
    let file_paths: Vec<String> = files
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();

    // Start with the arguments for exiftool
    // -g2: Group tags by family 2 (more specific groups like Camera, Image, Location)
    let mut args: Vec<&str> = vec!["-g2"];

    // Add file paths
    args.extend(file_paths.iter().map(|s| s.as_str()));

    // Execute exiftool on the sampled files
    println!("Running exiftool...");
    let mut tool = ExifTool::new()?;
    let exif_data_array = tool.execute_json(&args)?;

    println!("\nCombining JSON results...");
    let combined_json = combine_exif_json(&exif_data_array)?;

    println!("\nWriting combined JSON to file: {}", output_file_path);

    // Create/overwrite the output file
    let output_file = File::create(output_file_path)?;

    // Write the JSON data prettily to the file
    // Use serde_json::to_writer_pretty for direct writing
    serde_json::to_writer_pretty(output_file, &combined_json)?;

    println!(
        "Successfully wrote combined JSON data to {}",
        output_file_path
    );

    Ok(()) // Indicate successful execution
}
