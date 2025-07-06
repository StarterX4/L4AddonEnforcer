// SPDX-License-Identifier: LGPL-3.0-only
use crate::*;

pub fn autoinstall_addon(
    addon_file: &String,
    verbose: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    let datapack = vpk_getdata::main(&addon_file, verbose)?;
    if datapack.title.is_empty() {
        let err = format!(
            "Unable to define the addon name. Please specify it manually."
        );
        eprintln!("{} {}", "Error:".red(), err);
        println!(
            "Type {} / {} for more information",
            "-h".blue(),
            "--help".blue()
        );
        return Err(Box::new(QuietErr(Some(err))));
    }
        let i = install_addon(&addon_file, &sanitize_filename::sanitize(&datapack.title.replace(" ", "_").replace("'", "").as_str()), verbose)?;
        Ok(i)
}

pub fn install_addon(
    addon_file: &str,
    name: &str,
    verbose: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    let gameinfo_orig_md5 = format!("586b3b0b39bc44ddfb07792b1932c479");

    // Require both arguments on installation
    if (name.is_empty() && !addon_file.is_empty()) || (!name.is_empty() && addon_file.is_empty()) {
        let err = format!("Both addon name and addon file path must be provided!");
        eprintln!("{} {}", "Error:".red(), err);
        return Err(Box::new(QuietErr(Some(err))));
    }
    // Validate addon name
    if
    /*name.is_empty() || */
    name.contains(char::is_whitespace)
        || name.contains('/')
        || name.contains('\\')
        || name.contains(':')
        || name.contains('*')
        || name.contains('?')
        || name.contains('"')
        || name.contains('<')
        || name.contains('>')
        || name.contains('|')
    {
        let err = format!(
            "Invalid addon name! \n\tName cannot be empty, contain whitespace, or special characters \n\tthat are known to cause problems with file managers or filesystems."
        );
        eprintln!("{} {}", "Error:".red(), err);
        return Err(Box::new(QuietErr(Some(err))));
    }

    // Validate addon file
    let addon_path = PathBuf::from(&addon_file);
    if var_os("DEBUG").is_some() {
        println!("{} {} {:?}", "[D]".blue(), "Addon path:".bold(), addon_path);
    }
    if !addon_path.is_file() {
        let err = format!("Invalid addon file path!");
        eprintln!("{} {}", "Error:".red(), err);
        return Err(Box::new(QuietErr(Some(err))));
    }

    // Locate the gameinfo.txt file
    let gameinfo_path = gameinfo_path(verbose)?;
    // Calculate the MD5 of the gameinfo.txt file
    let gameinfo_md5 = calculate_md5(&gameinfo_path).expect("Failed to calculate MD5");

    let gameinfo_backup_path = gameinfo_backup_path(verbose)?;
    if var_os("DEBUG").is_some() || verbose {
        println!(
            "{} {} {:?}",
            "[D]".blue(),
            "gameinfo_backup_path:".bold(),
            gameinfo_backup_path
        );
    }
    if !gameinfo_backup_path.exists() {
        if gameinfo_md5 != gameinfo_orig_md5 {
            println!(
                "{} gameinfo.txt file seems to be modified!",
                "Warning:".yellow()
            );
            println!("The backup may be useless. Though proceeding anyway...");
            println!("");
            println!("If you haven't already modified the gameinfo.txt, this is probably");
            println!(
                "\ta {}'s bug you can report to the dev!",
                env!("CARGO_PKG_NAME")
            );
            println!("\t\tYour gameinfo.txt MD5 hash is: {}", gameinfo_md5.bold());
        }
        // Create a backup of the gameinfo.txt file
        copy(&gameinfo_path, &gameinfo_backup_path)?;
    } else {
        if var_os("DEBUG").is_some() || verbose {
            println!(
                "{} Found a gameinfo.txt backup ({:?})",
                "[D]".blue(),
                gameinfo_backup_path.file_name().unwrap().to_string_lossy()
            );
        }
    }

    // Read the gameinfo.txt file
    let contents = read_to_string(&gameinfo_path)?;

    // Split the file into lines
    let mut lines: Vec<&str> = contents.lines().collect();

    // Create the new addon directory
    if var_os("DEBUG").is_some() || verbose {
        println!("{} Creating addon directory: {}", "[D]".blue(), name);
    }
    let l4d2_dir = l4d2_path()?;
    let addon_dir = l4d2_dir.join(format!("{}", name));
    let mut addon_dir_existed = false;
    if addon_dir.exists() {
        addon_dir_existed = true;
        if var_os("DEBUG").is_some() || verbose {
            println!(
                "{} {} An addon directory with the same name already exists! Assuming this is an update.",
                "[D]".blue(),
                "Warning:".yellow()
            );
        }
    } else {
        create_dir_all(&addon_dir)?;
    }

    // Copy the addon file to the new addon directory
    if var_os("DEBUG").is_some() || verbose {
        println!("{} Copying addon file to: {:?}", "[D]".blue(), addon_dir);
    }
    let mut destination = addon_dir.join(addon_path.file_name().unwrap());
    destination.set_file_name("pak01_dir");
    destination.set_extension("vpk");
    if destination.exists() {
        remove_file(&destination)?;
        if var_os("DEBUG").is_some() || verbose {
            println!("{} Deleted {:?}", "[D]".blue(), destination);
        }
    }
    copy(&addon_path, &destination)?;

    // The line to insert
    let new_line = format!("\t\t\tGame\t\t\t\t{}", name);

    let mut up = false;
    if lines
        .iter()
        .any(|line| line.contains("Game") && line.contains(&name))
        && addon_dir_existed
    {
        println!("Updated {} successfully.", name.italic());
        up = true;
    } else {
        // Find the line with "Game update"
        let index = lines
            .iter()
            .position(|line| line.contains("Game") && line.contains("update"))
            .unwrap_or(0);

        // Insert the new line above it
        lines.insert(index, &new_line);

        // Join the lines back into a single string
        let new_contents = lines.join("\n");

        // Write the updated contents back to the file
        write(&gameinfo_path, new_contents)?;
        println!("Installed {} successfully.", name.italic());
    }
    if up { Ok(2) } else { Ok(1) }
}
