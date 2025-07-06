use clap::{Parser, Subcommand};
use seal_cli::check::check;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "seal")]
#[command(about = "Seal type checker")]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	Check {
		#[arg(help = "TypeScript file to check")]
		file: PathBuf,
	},
}

fn main() {
	let cli = Cli::parse();

	match cli.command {
		Commands::Check { file } => {
			if let Err(e) = check_file(file) {
				eprintln!("Error: {}", e);
				std::process::exit(1);
			}
		}
	}
}

fn check_file(file: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
	let source = std::fs::read_to_string(&file)?;

	let result = check(&source);
	
	if result.errors.is_empty() {
		println!("✓ Type checking passed for {}", file.display());
		Ok(())
	} else {
		eprintln!("Type checking failed for {}:", file.display());
		for error in &result.errors {
			eprintln!("  Line {}:{}: {}", error.start_line, error.start_column, error.message);
		}
		Err("Type checking failed".into())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::io::Write;
	use tempfile::NamedTempFile;

	#[test]
	fn test_check_file_valid() {
		let mut test_file = NamedTempFile::new().unwrap();
		writeln!(test_file, "const x: number = 42;").unwrap();

		let result = check_file(test_file.path().to_path_buf());
		assert!(result.is_ok());
	}

	#[test]
	fn test_check_file_invalid() {
		let mut test_file = NamedTempFile::new().unwrap();
		writeln!(test_file, "return 42;").unwrap(); // return outside function

		let result = check_file(test_file.path().to_path_buf());
		assert!(result.is_err());
	}

	#[test]
	fn test_check_file_nonexistent() {
		let result = check_file("/nonexistent/file.ts".into());
		assert!(result.is_err());
	}

	#[test]
	fn test_check_file_parse_error() {
		let mut test_file = NamedTempFile::new().unwrap();
		writeln!(test_file, "const x: = 42;").unwrap(); // syntax error

		let result = check_file(test_file.path().to_path_buf());
		assert!(result.is_err());
	}
}
