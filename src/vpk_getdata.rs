// SPDX-License-Identifier: LGPL-3.0-only

#![allow(non_snake_case)]
use sourcepak::common::file::VPKFileReader;
use sourcepak::{
    common::format::{PakReader, VPKDirectoryEntry},
    pak::v1::format::VPKVersion1,
};
use std::{fs::File, io::{Seek, SeekFrom}, path::Path, error::Error};

//read_single_file_vpk_v1
pub fn main(
    addon_file: &str,
    //verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(addon_file);
    let mut file = File::open(path)?;
    let vpk = VPKVersion1::try_from(&mut file).expect("Failed to read VPK file");

    let len = vpk.tree.files.len();
    println!("{:?}", len);

    // The key for "addoninfo.txt" at the root of the VPK is likely " /addoninfo.txt"
    // because the root path is a space, and sourcepak builds keys as "{path}/{file_name}.{extension}".
    let addoninfo_key = " /addoninfo.txt";
    let entry = vpk.tree.files.get(addoninfo_key);
    println!("{:?}", entry);

    let archive_dir = path.parent().unwrap_or_else(|| Path::new(".")).to_string_lossy();
    let vpk_name = path.file_stem().unwrap_or_default().to_string_lossy();
    let base_vpk_name = vpk_name.strip_suffix("_dir").unwrap_or(&vpk_name);

    // The sourcepak::read_file function fails silently by returning None, likely due to an
    // I/O error after an incorrect seek. We can work around this by manually reading the
    // data for entries stored in the main _dir.vpk file (archive_index == 0x7FFF).
    let test_file = entry.and_then(|e| {
        if e.archive_index == 0x7FFF {
            // For VPK v1, the tree starts immediately after the header. The data block for
            // embedded files starts immediately after the tree. The header's size is the
            // tree's offset from the start of the file.
            let tree_offset = std::mem::size_of_val(&vpk.header) as u64;
            let seek_pos = tree_offset + vpk.header.tree_size as u64 + e.entry_offset as u64;
            file.seek(SeekFrom::Start(seek_pos)).ok()?;
            file.read_bytes(e.entry_length as usize).ok()
        } else {
            // Fallback to sourcepak for other archive types (e.g., pak01_001.vpk)
            vpk.read_file(&archive_dir.to_string(), &base_vpk_name.to_string(), &addoninfo_key.to_string())
        }
    });

    println!("{:?}", test_file.clone().map(|bytes| String::from_utf8_lossy(&bytes).into_owned()));

    // Convert the bytes to a string, handling potential UTF-8 errors,
    // and return an error if the file content could not be read.
    let content = test_file
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .ok_or_else(|| {
            Box::<dyn Error>::from(format!("Failed to read addoninfo.txt content from VPK: {}", addon_file))
        })?;

    // Initialize variables to store the extracted data
    let mut title: Option<String> = None;
    let mut version: Option<String> = None;
    let mut description: Option<String> = None;

    // Iterate over each line of the content and extract the desired fields
    for line in content.lines() {
        if let Some(val) = extract_value(line, "()addonTitle") {
            title = Some(val);
        } else if let Some(val) = extract_value(line, "addonVersion") {
            version = Some(val);
        } else if let Some(val) = extract_value(line, "addonDescription") {
            description = Some(val);
        }
    }

    // Print the extracted information
    println!("Addon Title: {}", title.unwrap_or_else(|| "N/A".to_string()));
    println!("Addon Version: {}", version.unwrap_or_else(|| "N/A".to_string()));
    println!("Addon Description: {}", description.unwrap_or_else(|| "N/A".to_string()));

    Ok(())
}

// Helper function to extract the string value from a KeyValue formatted line.
// It looks for a key like `"key"` and then extracts the string within the next pair of quotes.
fn extract_value(line: &str, key: &str) -> Option<String> {
    let search_str = format!("\"{}\"", key);
    if let Some(start_of_key) = line.find(&search_str) {
        // Find the position right after the key's closing quote
        let after_key_start = start_of_key + search_str.len();
        // Search for the opening quote of the value string
        if let Some(value_start_quote_idx) = line[after_key_start..].find('"') {
            let actual_value_start = after_key_start + value_start_quote_idx + 1;
            // Search for the closing quote of the value string
            if let Some(value_end_quote_idx) = line[actual_value_start..].find('"') {
                // Extract the substring between the quotes
                return Some(line[actual_value_start..actual_value_start + value_end_quote_idx].to_string());
            }
        }
    }
    None // Return None if the key or its value couldn't be found
}