// SPDX-License-Identifier: LGPL-3.0-only

#![allow(non_snake_case)]
#![windows_subsystem = "windows"]
mod core_imports;
use crate::{core_args::SubCommands, core_imports::*};
mod core_args;

mod gui;
mod gui_theming;
mod install_addon;
mod rename_addon;
mod list_addons;
mod uninstall_addon;
mod pug_mode;
// mod gameinfo_reset;
mod vpk_getdata;




fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = core_args::Args::parse();
	match &args.command {
		Some(SubCommands::Install(install_args)) | Some(SubCommands::I(install_args)) => {
			// Install or update logic
			match (&install_args.file, &install_args.name) {
				(Some(addon_file), None) => {
					let _ = install_addon::autoinstall_addon(&addon_file, args.verbose);
				},
				(Some(addon_file), Some(name)) => {
					let _ = install_addon::install_addon(&addon_file, &name, args.verbose);
				},
				(None, Some(..)) => {
					let err = format!(
						"Argument -n <name> has been passed but no -f <file> have been passed!
											\nNo addon.vpk file provided for installation."
					);
					eprintln!("{} {}", "Error:".red(), err);
					println!(
						"Type {} / {} for more information",
						"-h".blue(),
						"--help".blue()
					);
					return Err(Box::new(QuietErr(Some(err))));
				},
				(None, None) => {
					let err = format!(
						"Arguments -n <name> and -f <file> have not been passed!
											\nNo addon.vpk file provided for installation."
					);
					eprintln!("{} {}", "Error:".red(), err);
					println!(
						"Type {} / {} for more information",
						"-h".blue(),
						"--help".blue()
					);
					return Err(Box::new(QuietErr(Some(err))));
				},
			}
		}
		Some(SubCommands::Uninstall(uninstall_args)) | Some(SubCommands::U(uninstall_args)) => {
			// Uninstall logic
			match &uninstall_args.name {
				Some(name) => {
					let _ = uninstall_addon::uninstall_addon(&name, args.verbose);
				},
				None => {
					let err = format!(
						"Argument -n <name> has not been passed!
											\nNo addon name provided for uninstalling."
					);
					eprintln!("{} {}", "Error:".red(), err);
					println!(
						"Type {} / {} for more information",
						"-h".blue(),
						"--help".blue()
					);
					return Err(Box::new(QuietErr(Some(err))));
				},
			}
		}
		Some(SubCommands::List(list_args)) | Some(SubCommands::L(list_args)) => {
			// List addons
			let _ = list_addons::list_addons(list_args.quiet, args.verbose, list_args.details, &mut std::io::stdout()); // Unused result becase of QuietErr usage?
		}
		Some(SubCommands::Rename(rename_args)) | Some(SubCommands::R(rename_args)) => {
			// Rename logic
			match (&rename_args.current, &rename_args.new) {
				(Some(current), Some(new)) => {
					let _ = rename_addon::rename_addon(&current, &new, args.verbose);
				},
				(Some(..), None) => {
					let err = format!(
						"Argument --current (-c) <current_name> has been passed but no --new (-n) <new_name> have been passed!
											\nNo new addon name provided for renaming."
					);
					eprintln!("{} {}", "Error:".red(), err);
					println!(
						"Type {} / {} for more information",
						"-h".blue(),
						"--help".blue()
					);
					return Err(Box::new(QuietErr(Some(err))));
				},
				(None, Some(..)) => {
					let err = format!(
						"Argument --current (-c) <current_name> has not been passed!
											\nNo currently installed addon provided for renaming."
					);
					eprintln!("{} {}", "Error:".red(), err);
					println!(
						"Type {} / {} for more information",
						"-h".blue(),
						"--help".blue()
					);
					return Err(Box::new(QuietErr(Some(err))));
				},
				(None, None) => {
					let err = format!(
						"Arguments --current (-c) <current_name> and --new (-n) <new_name> have not been passed!
											\nNo currently installed addon provided for renaming."
					);
					eprintln!("{} {}", "Error:".red(), err);
					println!(
						"Type {} / {} for more information",
						"-h".blue(),
						"--help".blue()
					);
					return Err(Box::new(QuietErr(Some(err))));
				},
			}
		}
		Some(SubCommands::Reset(reset_args)) | Some(SubCommands::Rs(reset_args)) => {
			// Reset logic
			if reset_args.confirm {
				let _ = gameinfo_reset(args.verbose); // Unused result becase of QuietErr usage?
			}
			else {
				let err = format!(
					"Argument --CONFIRM has not been passed!
										\nNo confirmation provided for reset."
				);
				eprintln!("{} {}", "Error:".red(), err);
				return Err(Box::new(QuietErr(Some(err))));
			}
			
		},
		Some(SubCommands::PuG(pug_args)) | Some(SubCommands::P(pug_args)) => {
			if pug_args.check {
				let _ = pug_mode::PuG_mode_check(args.verbose);
			} else if pug_args.switch {
				let _ = pug_mode::PuG_mode_switch(args.verbose);
			} else {
				let err = format!(
					"No PuG mode action specified!
										\nUse --check (-c) to check PuG mode status or --switch (-s) to toggle it."
				);
				eprintln!("{} {}", "Error:".red(), err);
				println!(
					"Type {} / {} for more information",
					"-h".blue(),
					"--help".blue()
				);
				return Err(Box::new(QuietErr(Some(err))));
			}
		}
		None => {
			if args.help {
				// Help logic
				print_short_help(true);
			} else if args.help_long {
				// Long help logic
				print_long_help(true);
			} else {
				gui::main();
			}
		},
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
impl Error for QuietErr {}

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

fn l4d2_path() -> Result<PathBuf, Box<dyn Error>> {
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
				let e = format!("{}: {}", err, err1);
				eprintln!("{} {}", "Error:".red(), e);
				Box::new(QuietErr(Some(e))) as Box<dyn Error>
			})?
			.find_app(550)
			.map_err(|_| {
				let e = format!("{}: {}", err, err2);
				eprintln!("{} {}", "Error:".red(), e);
				Box::new(QuietErr(Some(e))) as Box<dyn Error>
			})?
			.ok_or_else(|| {
				eprintln!("{} {}", "Error:".red(), err);
				Box::new(QuietErr(Some(err))) as Box<dyn Error>
			})?;
		Ok(library.resolve_app_dir(&app))
	}
}

fn gameinfo_path(verbose: bool) -> Result<PathBuf, Box<dyn std::error::Error>> {
	// Locate the Left 4 Dead 2 directory
	if var_os("DEBUG").is_some() || verbose {
		println!("{} Locating L4D2 directory...", "[D]".blue());
	}
	let l4d2_dir = l4d2_path()?;
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
	["A gameinfo.txt-based addon manager for Left 4 Dead 2."]
	["Use " c:"-h" " for short descriptions and " c:"--help" " for more details."]
	 [c:"--help" " for more details."]
	[]
	"USAGE" {
		[{env!("CARGO_PKG_NAME")} " [OPTIONS] [<SUBCOMMAND>] [<ARGS>]"]
	}
	"OPTIONS" {
		table Auto {
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
		}
	}
	"SUBCOMMANDS" {
		table Auto {
			"install, i" => {
				["Install an addon"]
				Long ["Installs or updates an addon. Requires either the path to the VPK file or \n"
					  "both the path and desired addon name."]
			}
			"list, l" => {
				["List installed addons"]
				Long ["Lists all currently installed addons."]
			}
			"uninstall, u" => {
				["Uninstall an addon"]
				Long ["Uninstalls a specified addon by name."]
			}
			"rename, r" => {
				["Rename an addon"]
				Long ["Renames an installed addon. Requires both the current and the new name."]
			}
			"pug, p" => {
				["Manage PuG mode"]
				Long ["Enables, disables, or checks the status of PuG mode."]
			}
			"reset, rs" => {
				["Reset gameinfo.txt"]
				Long ["Resets the gameinfo.txt file to its original state using a backup."]
			}
		}
	}
	"INSTALL SUBCOMMAND ARGS" {
		table Auto {
			"-f, --file <FILE_PATH>" => {
				["Path to the VPK addon file"]
			}
			"-n, --name <NAME>" => {
				["Specify the addon name manually"]
				Long ["If not provided, the addon name will be extracted from the VPK."]
			}
		}
	}
	"LIST SUBCOMMAND ARGS" {
		table Auto {
			"-d, --details" => {
				["List details for each addon (title, version, description"]
			}
		}
	}
	"UNINSTALL SUBCOMMAND ARGS" {
		table Auto {
			"-n, --name <NAME>" => {
				["Name of the addon to uninstall (optional)"]
			}
		}
	}
	"RENAME SUBCOMMAND ARGS" {
		table Auto {
			"-c, --current <CURRENT_NAME>" => {
				["Current name of the addon"]
			}
			"-n, --new <NEW_NAME>" => {
				["New name for the addon"]
			}
		}
	}
	"PUG SUBCOMMAND ARGS" {
		table Auto {
			"-c, --check" => {
				["Check PuG mode status"]
			}
			"-s, --switch" => {
				["Toggle PuG mode (enable/disable)"]
			}
		}
	}
	"RESET SUBCOMMAND ARGS" {
		table Auto {
			"--CONFIRM" => { ["Confirm the reset operation"] }
		}
	}
	[]
	"EXAMPLE" {
		["\t" g:{env!("CARGO_PKG_NAME")} " " c:"i" " " c:"/home/user/Downloads/ion_vocalizer.vpk"]
		[g:{env!("CARGO_PKG_NAME")} " " c:"i" " " C:"-f" " " c:"/home/user/Downloads/ion_vocalizer.vpk" " " C:"-n" " " c:"vocalizer"]
		[g:{env!("CARGO_PKG_NAME")} " " c:"u" " " C:"-n" " " c:"vocalizer"]
		[g:{env!("CARGO_PKG_NAME")} " " c:"r" " " C:"-c" " " c:"vocalizer" " " C:"-n" " " c:"ion_vocalizer"]
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
