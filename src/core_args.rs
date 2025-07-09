use crate::*;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
	// Subcommands (or groups of args, e.g. `./L4AddonEnforcer install -f FILE -n NAME`)
	#[command(subcommand)]
	pub command: Option<SubCommands>,

	// Non-grouped args (e.g. `./L4AddonEnforcer -p` to switch PuG mode)
	#[arg(short, long)] // TODO
	pub quiet: bool,
	#[arg(short, long)]
	pub verbose: bool,
	#[arg(short)]
	pub help: bool,
	#[arg(long = "help")]
	pub help_long: bool,
}

// Subcommands
#[derive(Parser, Debug)]
pub enum SubCommands {
	// Install an addon either manually or through name extraction (from provided VPK)
	Install(InstallArgs),
	I(InstallArgs),
	// List currently installed addons
	List(ListArgs),
	L(ListArgs),
	// Uninstall an addon
	Uninstall(UninstallArgs),
	U(UninstallArgs),
	// Rename an addon
	Rename(RenameArgs),
	R(RenameArgs),
	// PuG mode management
	PuG(PuGArgs),
	P(PuGArgs),
	// Reset gameinfo.txt using the backup
	Reset(ResetArgs),
	Rs(ResetArgs),
}

// Arguments for the `install` subcommand
#[derive(Parser, Debug)]
pub struct InstallArgs {
	#[arg(short, long, value_name = "FILE_PATH", num_args = 1.., index = 1)]
	pub file: Option<String>,

	#[arg(short, long, value_name = "NAME")]
	pub name: Option<String>,

	#[arg(short, long)] // TODO
	pub quiet: bool,

	#[arg(short, long)]
	pub verbose: bool,
}

// Arguments for the `list` subcommand
#[derive(Parser, Debug)]
pub struct ListArgs {
	#[arg(short, long)]
	pub details: bool,

	#[arg(short, long)]
	pub quiet: bool,
	
	#[arg(short, long)]
	 pub verbose: bool,
}

// Arguments for the `uninstall` subcommand
#[derive(Parser, Debug)]
pub struct UninstallArgs {
	#[arg(short, long, value_name = "NAME", num_args = 1.., index = 1)]
	pub name: Option<String>,
	
	#[arg(short, long)] // TODO
	pub quiet: bool,

	#[arg(short, long)]
	pub verbose: bool,
}

// Arguments for the `rename` subcommand
#[derive(Parser, Debug)]
pub struct RenameArgs {
	#[arg(short, long, value_name = "CURRENT_NAME")]
	pub current: Option<String>,

	#[arg(short, long, value_name = "NEW_NAME")]
	pub new: Option<String>,
	
	// #[arg(short, long)] // TODO
	// pub quiet: bool,

	#[arg(short, long)]
	pub verbose: bool,
}

// Arguments for the `pug` subcommand
#[derive(Parser, Debug)]
pub struct PuGArgs {
	#[arg(short, long)]
	pub check: bool,

	#[arg(short, long)]
	pub switch: bool,

	#[arg(short, long)]
	pub verbose: bool,
}

// Arguments for the `reset` subcommand
#[derive(Parser, Debug)]
pub struct ResetArgs {
	#[arg(long = "CONFIRM")]
	pub confirm: bool,

	#[arg(short, long)]
	pub verbose: bool,
}