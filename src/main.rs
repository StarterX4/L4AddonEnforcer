// SPDX-License-Identifier: LGPL-3.0-only

#![allow(non_snake_case)]
#![windows_subsystem = "windows"]
mod core_imports;
use crate::core_imports::*;

mod gui;
mod gui_theming;
mod install_addon;
mod rename_addon;
mod list_addons;
mod uninstall_addon;
mod pug_mode;
// mod gameinfo_reset;
mod vpk_getdata;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE_PATH")]
    auto: Option<String>,
    #[arg(short, long, value_name = "FILE_PATH")]
    file: Option<String>,
    #[arg(short, long, value_name = "NAME")]
    name: Option<String>,
    #[arg(short, long, value_name = "NAME")]
    uninstall: Option<String>,
    #[arg(short, long)]
    list: bool,
    #[arg(short, long)] // TODO
    quiet: bool,
    #[arg(short = 'p', long = "pug")]
    pug_switch: bool,
    #[arg(short = 'P', long = "checkpug")]
    pug_check: bool,
    #[arg(short, long)]
    rename: Option<String>,
    #[arg(long)]
    reset: bool,
    #[arg(short, long)]
    verbose: bool,
    #[arg(short)]
    help: bool,
    #[arg(long = "help")]
    help_long: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if (args.file.is_none() && args.rename.is_none()) && args.name.is_some() {
        let err = format!(
            "Argument -n <name> has been passed but no -f <file> or -r <old_name> have been passed!
                                \nNo addon.vpk file provided for installation
                                \nNor a currently installed addon for renaming."
        );
        eprintln!("{} {}", "Error:".red(), err);
        println!(
            "Type {} / {} for more information",
            "-h".blue(),
            "--help".blue()
        );
        return Err(Box::new(QuietErr(Some(err))));
    }

    if let Some(addon_file) = args.file {
        // Install or update logic
        if let Some(name) = args.name {
            let _ = install_addon::install_addon(&addon_file, &name, args.verbose); // Unused result becase of QuietErr usage?
        } else {
            eprintln!("{} No addon name provided for installation", "Error:".red());
            println!(
                "Type {} / {} for more information",
                "-h".blue(),
                "--help".blue()
            );
            exit(22)
        }
    } else if let Some(name) = args.uninstall {
        // Uninstall logic
        let _ = uninstall_addon::uninstall_addon(&name, args.verbose); // Unused result becase of QuietErr usage?
    } else if args.list {
        // List addons
        let _ = list_addons::list_addons(args.quiet, args.verbose, &mut std::io::stdout()); // Unused result becase of QuietErr usage?
    } else if args.pug_switch {
        // PuG mode switch logic
        let _ = pug_mode::PuG_mode_switch(args.verbose);
    } else if args.pug_check {
        // PuG mode check logic
        let _ = pug_mode::PuG_mode_check(args.verbose);
    } else if let Some(ren) = args.rename {
        // Rename logic
        if let Some(name) = args.name {
            let _ = rename_addon::rename_addon(&ren, &name, args.verbose); // Unused result becase of QuietErr usage?
        } else {
            eprintln!("{} No new addon name provided for renaming", "Error:".red());
            println!(
                "Type {} / {} for more information",
                "-h".blue(),
                "--help".blue()
            );
            exit(22)
        }
    } else if args.reset {
        // Gameinfo.txt reset logic
        let _ = gameinfo_reset(args.verbose); // Unused result becase of QuietErr usage?
    } else if args.help {
        // Help logic
        print_short_help(true);
    } else if args.help_long {
        // Long help logic
        print_long_help(true);
    } else if let Some(addonpath) = args.auto {
        let _ = vpk_getdata::main(&addonpath, args.verbose);
    } else {
        gui::main();
    }

    Ok(())
}

fn gameinfo_reset(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let gameinfo_orig_md5 = format!("586b3b0b39bc44ddfb07792b1932c479");
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
                "{} gameinfo.txt file seems to be modified, but no backup is present!",
                "Warning:".yellow()
            );
            println!("");
            println!("If you haven't already modified the gameinfo.txt, this is probably");
            println!(
                "\ta {}'s bug you can report to the dev!",
                env!("CARGO_PKG_NAME")
            );
            println!("\t\tYour gameinfo.txt MD5 hash is: {}", gameinfo_md5.bold());
        } else {
            let err = format!("gameinfo.txt is already at its default state!");
            eprintln!("{} {}", "Error:".red(), err);
            return Err(Box::new(QuietErr(Some(err))));
        }
    } else {
        if gameinfo_md5 != gameinfo_orig_md5 {
            if var_os("DEBUG").is_some() || verbose {
                println!(
                    "{} Copying gameinfo.txt backup ({:?}) to {:?}",
                    "[D]".blue(),
                    &gameinfo_backup_path.file_name().unwrap().to_string_lossy(),
                    &gameinfo_path.file_name().unwrap().to_string_lossy()
                );
            }
            copy(&gameinfo_backup_path, &gameinfo_path)?;
            println!("Succesfully reset gameinfo.txt to default.");
        } else {
            let err = format!("gameinfo.txt is already at its default state!");
            eprintln!("{} {}", "Error:".red(), err);
            return Err(Box::new(QuietErr(Some(err))));
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct QuietErr(Option<String>);
impl fmt::Display for QuietErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref msg) = self.0 {
            write!(f, "{}", msg)
        } else {
            write!(f, "")
        }
    }
}
impl std::error::Error for QuietErr {}

fn calculate_md5(filepath: &Path) -> Result<String, std::io::Error> {
    let mut file = BufReader::new(File::open(filepath)?); // Buffered for performance
    let mut hasher = Md5::new();
    let mut buffer = [0; 4096]; // Read in chunks for efficiency
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let result = hasher.finalize();
    Ok(format!("{:x}", result)) // Format as hexadecimal string
}

// fn clean_path(path: &str) -> Cow<str> {
//     if path.contains("/../") {
//         let path_buf = PathBuf::from(format!("/{path}"));
//         let Ok(absolute_path) = path_buf.parse_dot_from("/") else {
//             return path.into();
//         };
//         let path = absolute_path.to_str().unwrap().trim_start_matches('/');
//         String::from(path).into()
//     } else {
//         path.into()
//     }
// }

// #[test]
// fn test_clean_path() {
//     assert_eq!("foo/bar", clean_path("foo/bar"));
//     assert_eq!("foo/bar", clean_path("foo/asd/../bar"));
//     assert_eq!("../bar", clean_path("../bar"));
// }

fn l4d2_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let err = "Failed to find L4D2 install location".to_string();
    let err1 = "Failed to find Steam installation location".to_string();
    let err2 = "Failed to locate Left 4 Dead 2 directory".to_string();
    if let Some(path) = var_os("L4D2_DIR") {
        let path: PathBuf = path.into();
        if path.is_dir() {
            Ok(path)
        } else {
            eprintln!("{} {}: {}", "Error:".red(), err, err1);
            let e = format!("\n{}: \n{}", err, err1);
            return Err(Box::new(QuietErr(Some(e))));
        }
    } else {
        let (app, library) = SteamDir::locate()
            .map_err(|_| {
                let e = format!("{}: {}", err.clone(), err1);
                eprintln!("{} {}", "Error:".red(), e);
                QuietErr(Some(e))
            })?
            .find_app(550)
            .map_err(|_| {
                let e = format!("{}: {}", err.clone(), err2);
                eprintln!("{} {}", "Error:".red(), e);
                QuietErr(Some(e))
            })?
            .ok_or({
                let e = err.clone();
                eprintln!("{} {}", "Error:".red(), e);
                QuietErr(Some(e))
            })?;
        Ok(library.resolve_app_dir(&app))
    }
}

fn gameinfo_path(verbose: bool) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Locate the Left 4 Dead 2 directory
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
    Ok(gameinfo_path)
}

fn gameinfo_backup_path(verbose: bool) -> Result<PathBuf, Box<dyn std::error::Error>> {
    //let backup_path = PathBuf::new();
    let l4d2_dir = l4d2_path()?;
    if let Some(path) = var_os("BACKUP_PATH") {
        if var_os("DEBUG").is_some() || verbose {
            println!(
                "{} {}={:?}",
                "[D]".blue(),
                "env BACKUP_PATH".bold(),
                path.to_string_lossy()
            );
        }
        let path: PathBuf = path.into();
        if path.is_file() {
            Ok(path)
        } else {
            let err = "is not a file".to_string();
            eprintln!("{} {} {}", "Error:".red(), path.to_string_lossy(), err);
            let e = format!("\n{} \n{}", path.to_string_lossy(), err);
            return Err(Box::new(QuietErr(Some(e))));
        }
    } else if let Some(name) = var_os("BACKUP_NAME") {
        if var_os("DEBUG").is_some() || verbose {
            println!(
                "{} {}={:?}",
                "[D]".blue(),
                "env BACKUP_NAME".bold(),
                name.to_string_lossy()
            );
        }
        let name = name.to_string_lossy().to_string();
        let place = format!("left4dead2/{}", name);
        let place_invalid = format!("left4dead2/");
        if place_invalid == place {
            let err = "the backup place[from name!] does not contain any file name!".to_string();
            eprintln!("{} {}  <- {}", "Error:".red(), place, err);
            let e = format!("\n{} \n^-- {} --^", place, err);
            return Err(Box::new(QuietErr(Some(e))));
        } else {
            let backup_name = l4d2_dir.join(place);
            if backup_name.exists() {
                Ok(backup_name)
            } else {
                let err = "the backup file does not exist!".to_string();
                eprintln!(
                    "{} {} — {}",
                    "Error:".red(),
                    backup_name.to_string_lossy(),
                    err
                );
                let e = format!("\n{} \n^-- {} --^", backup_name.to_string_lossy(), err);
                Err(Box::new(QuietErr(Some(e))))
            }
        }
    } else {
        if var_os("DEBUG").is_some() || verbose {
            println!(
                "{} {}",
                "[D]".blue(),
                "No custom backup path is provided. Using default 'gameinfo.txt.orig'".bold(),
            );
        }
        let backup_def_path = l4d2_dir.join("left4dead2/gameinfo.txt.orig");
        Ok(backup_def_path)
    }
}

const HELP: Help = Help(sections!(
    [{env!("CARGO_PKG_NAME")} " " {env!("CARGO_PKG_VERSION")}]
    ["Use " c:"-h" " for short descriptions and " c:"--help" " for more details."]
    []
    "USAGE" {
        [{env!("CARGO_PKG_NAME")} " [OPTIONS] [<ARGS>]"]
    }
    "OPTIONS" {
        table Auto {
            "-f, --file <FILE>" => {
                ["VPK Addon file path"]
            }
            "-l, --list" => {
                ["List currently installed addons"]
            }
            "-n, --name <NAME>" => {
                ["Name for the addon that will be installed/updated or renamed (if already installed)"]
                Long ["a directory that will be named after," "and an entry in the gameinfo.txt.\n"
                "You can name it whatever you like, to later know what that addon is."]
            }
            "-u, --uninstall <NAME>" => {
                ["Uninstall the already installed addon"]
            }
            "-r, --rename <CURRENT_NAME>" => {
                ["Rename the already installed addon"]
            }
            "-p, --pug" => {
                ["Switches the PuG Mode (on/off)."]
                Long ["Temporarily backup the current gameinfo.txt and restore the vanilla one.\n"
                "Or vice versa.\n"
                "Useful for competitive PuG-type servers that enforce file consistency."]
            }
            "-P, --checkpug" => {
                ["Checks the PuG Mode status."]
            }
            "--reset" => {
                ["Reset the current gameinfo.txt to its default state using the existing backup"]
            }
            "-h, --help" => {
                ["Print help information"]
                Long ["Use " c:"-h" " for short descriptions and " c:"--help" " for more details."]
            }
            "-v, --verbose" => {
                ["Enable verbose output"]
                Long ["showing more details about the operations being performed."]
            }
            "-V, --version" => {
                ["Print version information"]
            }
//             "-W, --warnings <DIAGNOSTICS>" => {
//                 Short ["Disable certain warnings (disable all with " c:"-W0" ")"]
//                 Long ["Disable some or all warnings. A single warning can be disabled by specifying
// the name followed by " c:"=0" ", for example:

//     " c!"-Wcompat=0" "

// Multiple warnings can be disabled by setting this option multiple times, or
// using a comma-separated list:

//     " c!"-Wcompat=0 -Wdeprecated=0
//     -Wcompat=0,deprecated=0" "

// To disable all warnings, use " c:"-W0" ".

// Currently, the following warnings can be disabled:"]
//                 Long table Compact {
//                     "compat"     => { ["Compatibility warnings"] }
//                     "deprecated" => { ["A used feature will be removed in the future"] }
//                 }
//             }
        }
    }
    "EXAMPLE" {
        ["\t" g:{env!("CARGO_PKG_NAME")} " " C:"-f" " " c:"/home/user/Downloads/ion_vocalizer.vpk" " " C:"-n" " " c:"vocalizer"]
        [g:{env!("CARGO_PKG_NAME")} " " C:"-u" " " c:"vocalizer"]
        [g:{env!("CARGO_PKG_NAME")} " " C:"-r" " " c:"vocalizer" " " C:"-n" " " c:"ion_vocalizer"]
    }
    "ENVIRONMENT VARIABLES" {
        [c:"\tL4D2_DIR\n\t"
            "Directory where the game is installed"
        ]
        []
        [c:"BACKUP_PATH\n\t"
            "Path to the backup file (either to be created in or to be used in restore)"
        ]
        []
        [c:"BACKUP_NAME\n\t"
            "Name of the backup file residing in the left4dead2 directory \n\t(either to be created as or to be used in restore)\n\t"
            "If this is set, the selected backup file name will be used.\n\t"
            "If this is not set, the default name will be used: " m:"gameinfo.txt.orig"
        ]
        []
        [c:"DEBUG\n\t"
            "Enables verbose output\n\t"
            "(Equivalent to the \"-v\" option)"
        ]
    }
));

fn print_short_help(use_colors: bool) {
    #![allow(unused_must_use)]
    HELP.write(
        &mut std::io::stdout().lock(),
        false, // don't show long help
        use_colors,
    );
}

fn print_long_help(use_colors: bool) {
    #![allow(unused_must_use)]
    HELP.write(
        &mut std::io::stdout().lock(),
        true, // show long help
        use_colors,
    );
}
