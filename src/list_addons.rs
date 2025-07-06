use std::{env::join_paths, path};

// SPDX-License-Identifier: LGPL-3.0-only
use crate::*;

pub fn list_addons(
    quiet: bool,
    verbose: bool,
    details: bool,
    buf_writer: &mut impl Write,
) -> Result<(), Box<dyn std::error::Error>> {
    // Locate the gameinfo.txt file
    let gameinfo_path = gameinfo_path(verbose)?;

    // Read the gameinfo.txt file
    let contents = read_to_string(&gameinfo_path)?;

    // Split the file into lines
    let lines: Vec<&str> = contents.lines().collect();

    let excl_addons = vec![
        "update",
        "left4dead2_dlc3",
        "left4dead2_dlc2",
        "left4dead2_dlc1",
        "hl2",
        "|gameinfo_path|.",
    ];

    // List the installed custom addons
    let mut SearchPaths = false;
    if lines.iter().any(|line| line.contains("SearchPaths")) {
        if !quiet {
            println!("{}", "Installed addons:".bold());
        }
        let mut l4d2_dir: PathBuf = path::PathBuf::new();
        if details {
            l4d2_dir = l4d2_path()?;
        }
        for line in lines.iter() {
            if line.contains("SearchPaths") {
                SearchPaths = true;
            } else if SearchPaths && line.contains("}") {
                SearchPaths = false;
            }
            if SearchPaths
                && line.contains("Game")
                && !excl_addons.iter().any(|&s| line.contains(s))
            {
                let addon = line
                    .trim_start_matches('\t')
                    .trim_start_matches("Game")
                    .replace("\t\t\t\t", "\t");
                if details {
                    let addon_file = l4d2_dir.join(&addon.trim_start_matches('\t')).join("pak01_dir.vpk");
                    if verbose {println!("{} path: {}", &addon.green(), addon_file.to_string_lossy().to_string().purple());}
                    if addon_file.exists() {
                        let datapack = vpk_getdata::main(&addon_file.to_string_lossy().to_string(), verbose)?;
                        writeln!(buf_writer, "{} (title: {}, version: {}, description: {})", addon, datapack.title, datapack.version, datapack.description).unwrap();
                    } else {
                        writeln!(buf_writer, "{}", addon).unwrap();
                    }
                } else {
                writeln!(buf_writer, "{}", addon).unwrap();
                }
            }
        }
        return Ok(());
    }
    Ok(())
}