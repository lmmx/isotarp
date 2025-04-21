use clap::Parser;
use isotarp::cli::{Cli, Commands, execute_analyze_command, execute_list_command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { package } => {
            execute_list_command(&package)?;
        }
        Commands::Analyze {
            package,
            tests,
            output_dir,
            report,
        } => {
            execute_analyze_command(&package, tests, &output_dir, &report)?;
        }
    }

    Ok(())
}
