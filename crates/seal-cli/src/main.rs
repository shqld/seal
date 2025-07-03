use clap::{Parser, Subcommand};
use seal_ty::{checker::Checker, context::TyContext, parse::parse};
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
    
    let parse_result = parse(&source).map_err(|e| format!("Parse error: {:?}", e))?;
    let tcx = TyContext::new();
    let checker = Checker::new(&tcx);
    
    match checker.check(&parse_result.program) {
        Ok(()) => {
            println!("✓ Type checking passed for {}", file.display());
            Ok(())
        }
        Err(errors) => {
            eprintln!("Type checking failed for {}:", file.display());
            for error in errors {
                eprintln!("  {:?}", error);
            }
            Err("Type checking failed".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_check_file_valid() {
        let test_file = "/tmp/test_valid.ts";
        fs::write(test_file, "const x: number = 42;").unwrap();
        
        let result = check_file(test_file.into());
        assert!(result.is_ok());
        
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_check_file_invalid() {
        let test_file = "/tmp/test_invalid.ts";
        fs::write(test_file, "return 42;").unwrap(); // return outside function
        
        let result = check_file(test_file.into());
        assert!(result.is_err());
        
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_check_file_nonexistent() {
        let result = check_file("/nonexistent/file.ts".into());
        assert!(result.is_err());
    }

    #[test]
    fn test_check_file_parse_error() {
        let test_file = "/tmp/test_parse_error.ts";
        fs::write(test_file, "const x: = 42;").unwrap(); // syntax error
        
        let result = check_file(test_file.into());
        assert!(result.is_err());
        
        fs::remove_file(test_file).ok();
    }
}