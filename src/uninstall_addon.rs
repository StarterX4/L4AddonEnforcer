// SPDX-License-Identifier: LGPL-3.0-only
use crate::*;

pub fn uninstall_addon(del_name: &str, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    if del_name.is_empty() {
        let err = format!("No addon name provided for uninstallation");
        eprintln!("{} {}", "Error:".red(), err);
        return Err(Box::new(QuietErr(Some(err))));
    }

    // Locate the gameinfo.txt file
    let gameinfo_path = gameinfo_path(verbose)?;

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
    if !del_name.is_empty() && !excl_addons.iter().any(|&s| del_name.contains(s)) {
        if lines
            .iter()
            .any(|line| line.contains("Game") && line.contains(&del_name))
        {
            let index = lines
                .iter()
                .position(|line| line.contains("Game") && line.contains(&del_name))
                .unwrap();
            lines.remove(index);
            let new_contents = lines.join("\n");
            write(&gameinfo_path, new_contents)?;
            if var_os("DEBUG").is_some() || verbose {
                println!(
                    "{} Removing line \n{} \nfrom gameinfo.txt",
                    "[D]".blue(),
                    index
                );
            }
        } else {
            let err = format!("{} not found in the gameinfo.txt file!", del_name);
            eprintln!("{} {}", "Error:".red(), err);
            return Err(Box::new(QuietErr(Some(err))));
        }
        let del_addon_dir = l4d2_path()?.join(format!("{}", del_name));
        if del_addon_dir.exists() {
            if del_addon_dir.is_dir() {
                std::fs::remove_dir_all(&del_addon_dir)?;
                println!("Uninstalled {} successfully.", del_name.italic());
            } else {
                if var_os("DEBUG").is_some() || verbose {
                    println!(
                        "{} {} appears to not be a directory! (filesystem damaged? installation failed?)",
                        "[D]".blue(),
                        del_addon_dir.display()
                    );
                }
            }
        }
        return Ok(());
    } else {
        if excl_addons.iter().any(|&s| del_name.contains(s)) {
            let err = format!("Core game components cannot be uninstalled!");
            eprintln!("{} {}", "Error:".red(), err);
            return Err(Box::new(QuietErr(Some(err))));
        }
    }
    Ok(())
}
