// SPDX-License-Identifier: LGPL-3.0-only
use crate::*;

pub fn rename_addon(
    ren_name: &str,
    new_name: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if ren_name.is_empty() || new_name.is_empty() {
        let err = format!("No addon names provided for renaming");
        eprintln!("{} {}", "Error:".red(), err);
        return Err(Box::new(QuietErr(Some(err))));
    }
    // Validate addon name
    if new_name.contains(char::is_whitespace)
        || new_name.contains('/')
        || new_name.contains('\\')
        || new_name.contains(':')
        || new_name.contains('*')
        || new_name.contains('?')
        || new_name.contains('"')
        || new_name.contains('<')
        || new_name.contains('>')
        || new_name.contains('|')
    {
        let err = format!(
            "Invalid addon name! \n\tNew name cannot be empty, contain whitespace, or special characters \n\tthat are known to cause problems with file managers or filesystems."
        );
        eprintln!("{} {}", "Error:".red(), err);
        return Err(Box::new(QuietErr(Some(err))));
    }

    // Locate the Left 4 Dead 2 directory ... LEAVE THIS WITH THE OLD WAY FOR NOW ...
    if var_os("DEBUG").is_some() || verbose {
        println!("{} Locating L4D2 directory...", "[D]".blue());
    }
    let l4d2_dir = l4d2_path().expect("Failed to locate Left 4 Dead 2 directory");
    let gameinfo_path = l4d2_dir.join("left4dead2/gameinfo.txt");

    if !gameinfo_path.exists() {
        let err = format!("Unable to locate gameinfo.txt file. Is the game installation broken?");
        eprintln!("{} {}", "Error:".red(), err);
        return Err(Box::new(QuietErr(Some(err))));
    }

    if var_os("DEBUG").is_some() || verbose {
        println!(
            "{} {} {:?}",
            "[D]".blue(),
            "Gameinfo.txt path:".bold(),
            gameinfo_path
        );
    }

    // Read the gameinfo.txt file
    let contents = read_to_string(&gameinfo_path)?;

    // Split the file into lines
    let mut lines: Vec<&str> = contents.lines().collect();

    let excl_addons = vec![
        "update",
        "left4dead2_dlc3",
        "left4dead2_dlc2",
        "left4dead2_dlc1",
        "hl2",
        "|gameinfo_path|.",
    ];

    // Delete the selected addon
    if !ren_name.is_empty()
        && !new_name.is_empty()
        && !excl_addons.iter().any(|&s| ren_name.contains(s))
        && !excl_addons.iter().any(|&s| new_name.contains(s))
    {
        if lines
            .iter()
            .any(|line| line.contains("Game") && line.contains(&new_name))
        {
            let err = format!(
                "{} is already installed! \n\t Please choose a different name.",
                new_name
            );
            eprintln!("{} {}", "Error:".red(), err);
            return Err(Box::new(QuietErr(Some(err))));
        }

        if lines
            .iter()
            .any(|line| line.contains("Game") && line.contains(&ren_name))
        {
            // The line to insert
            let new_line = format!("\t\t\tGame\t\t\t\t{}", new_name);
            // Find the line with "Game {ren_name}"
            let index = lines
                .iter()
                .position(|line| line.contains("Game") && line.contains(&ren_name))
                .unwrap();
            lines.remove(index);
            if var_os("DEBUG").is_some() || verbose {
                println!(
                    "{} Removing line \n{} \nfrom gameinfo.txt",
                    "[D]".blue(),
                    index
                );
            }
            // Insert the new line above it
            lines.insert(index, &new_line);
            if var_os("DEBUG").is_some() || verbose {
                println!(
                    "{} Inserting line \n{} \n in the line \n {} \n in gameinfo.txt",
                    "[D]".blue(),
                    new_line,
                    index
                );
            }
            let new_contents = lines.join("\n");
            write(&gameinfo_path, new_contents)?;
        } else {
            let err = format!("{} not found in the gameinfo.txt file!", ren_name);
            eprintln!("{} {}", "Error:".red(), err);
            return Err(Box::new(QuietErr(Some(err))));
        }
        let ren_addon_dir = l4d2_dir.join(format!("{}", ren_name));
        let ren_new_addon_dir = l4d2_dir.join(format!("{}", new_name));
        if ren_addon_dir.exists() {
            if ren_addon_dir.is_dir() {
                std::fs::rename(&ren_addon_dir, &ren_new_addon_dir)?;
                println!(
                    "Renamed {} to {} successfully.",
                    ren_name.italic(),
                    new_name.italic()
                );
            } else {
                if var_os("DEBUG").is_some() || verbose {
                    println!(
                        "{} {} appears to not be a directory! (filesystem damaged? installation failed?)",
                        "[D]".blue(),
                        ren_addon_dir.display()
                    );
                }
            }
        }
        return Ok(());
    } else {
        if excl_addons.iter().any(|&s| ren_name.contains(s)) {
            let err = format!(
                "Core game components cannot be renamed! \n\t Found: \"{}\"",
                ren_name
            );
            eprintln!("{} {}", "Error:".red(), err);
            return Err(Box::new(QuietErr(Some(err))));
        }
        if excl_addons.iter().any(|&s| new_name.contains(s)) {
            let err = format!(
                "New name \"{}\" conflicts with one of the core game components!",
                new_name
            );
            eprintln!("{} {}", "Error:".red(), err);
            return Err(Box::new(QuietErr(Some(err))));
        }
    }
    Ok(())
}