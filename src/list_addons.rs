// SPDX-License-Identifier: LGPL-3.0-only
use crate::*;

pub fn list_addons(
    quiet: bool,
    verbose: bool,
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
                writeln!(buf_writer, "{}", addon).unwrap();
            }
        }
        return Ok(());
    }
    Ok(())
}