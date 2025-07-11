// SPDX-License-Identifier: LGPL-3.0-only

#![allow(non_snake_case)]
use sourcepak::common::file::VPKFileReader;
use sourcepak::{
	common::format::{PakReader, VPKDirectoryEntry},
	pak::v1::format::VPKVersion1,
};
use std::env::var_os;
use std::{fs::File, io::{Seek, SeekFrom}, path::Path, error::Error};
use regex::Regex;

pub struct ExtractedData {
	pub title: String,
	pub version: String,
	pub description: String,
}

//read_single_file_vpk_v1
pub fn main(
	addon_file: &String,
	verbose: bool,
) -> Result<ExtractedData, Box<dyn std::error::Error>> {
	let path = Path::new(addon_file);
	let mut file = File::open(path)?;
	let vpk = VPKVersion1::try_from(&mut file).expect("Failed to read VPK file");

	// Need to know the number of entries (files and directories) in the vpk?
	if var_os("DEBUG").is_some() || verbose {
		let len = vpk.tree.files.len();
		println!("{:?}", len);
	}

	// The key for "addoninfo.txt" at the root of the VPK is likely " /addoninfo.txt"
	// because the root path is a space, and sourcepak builds keys as "{path}/{file_name}.{extension}".
	let addoninfo_key = " /addoninfo.txt";
	let entry = vpk.tree.files.get(addoninfo_key);
	// Print out the addoninfo
	if var_os("DEBUG").is_some() || verbose {
		println!("{:?}", entry);
	}

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
		if let Some(val) = extract_value(line, "addonTitle") {
			title = Some(val);
		} else if let Some(val) = extract_value(line, "addonVersion") {
			version = Some(val);
		} else if let Some(val) = extract_value(line, "addonDescription") {
			description = Some(val);
		}
	}

	// Print the extracted information
	// if var_os("DEBUG").is_some() || verbose {
	// println!("Addon Title: {}", title.clone().unwrap_or_else(|| "N/A".to_string()));
	// println!("Addon Version: {}", version.clone().unwrap_or_else(|| "N/A".to_string()));
	// println!("Addon Description: {}", description.clone().unwrap_or_else(|| "N/A".to_string()));
	// }

	Ok(ExtractedData{
		title: title.unwrap_or_else(|| "N/A".to_string()).to_string(),
		version: version.unwrap_or_else(|| "N/A".to_string()).to_string(),
		description: description.unwrap_or_else(|| "N/A".to_string()).to_string(),
	})
}

// Helper function to extract the string value from a KeyValue formatted line.
// It uses a regular expression for case-insensitive key matching.
fn extract_value(line: &str, key: &str) -> Option<String> {
	// The regex looks for a key (case-insensitive) followed by a value, all in quotes.
	// It handles both quoted ("addonTitle") and unquoted (addonTitle) keys.
	// The (?i) flag makes the key match case-insensitive.
	// \s+ matches one or more whitespace characters between key and value.
	// "([^"]*)" captures the value string.
	let pattern = format!(r#"^\s*(?i)(?:"{0}"|{0})\s+"([^"]*)""#, regex::escape(key));
	// It's safe to unwrap here as we control the regex string.
	let re = Regex::new(&pattern).unwrap();

	// `captures` returns an Option, and we can map over it.
	re.captures(line)
		.and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}