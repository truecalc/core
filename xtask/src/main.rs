mod gen_array_filter;
mod gen_database;
mod gen_lookup;
mod gen_math;
mod gen_statistical;
mod gen_text_date_eng_fin;
mod generate;
mod types;

use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand, ValueEnum};

use types::Platform;

#[derive(Debug, Clone, ValueEnum)]
enum PlatformArg {
    Sheets,
    Excel,
}

impl PlatformArg {
    fn to_platform(&self) -> Platform {
        match self {
            PlatformArg::Sheets => Platform::Sheets,
            PlatformArg::Excel => Platform::Excel,
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "xtask", about = "Fixture generation and oracle evaluation tasks")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Generate pre-oracle fixture TSV files
    GenerateFixtures {
        #[arg(long)]
        platform: PlatformArg,
        /// Generate fixtures for a specific category
        #[arg(long)]
        category: Option<String>,
        /// Generate fixtures for all categories
        #[arg(long)]
        all: bool,
    },
    /// Evaluate fixtures against the oracle (stub — see T3.8)
    OracleEvaluate {
        #[arg(long)]
        platform: PlatformArg,
        /// Evaluate all categories
        #[arg(long)]
        all: bool,
        /// Evaluate a specific category
        #[arg(long)]
        category: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::GenerateFixtures { platform, category, all } => {
            if !all && category.is_none() {
                bail!("Specify --all or --category <name>");
            }
            run_generate_fixtures(platform.to_platform(), category.as_deref(), all)?;
        }
        Commands::OracleEvaluate { .. } => {
            println!("not yet implemented — see T3.8");
        }
    }

    Ok(())
}

fn run_generate_fixtures(platform: Platform, category: Option<&str>, all: bool) -> Result<()> {
    let out_dir = PathBuf::from("target/fixture-gen").join(platform.dir_name());
    std::fs::create_dir_all(&out_dir)?;

    let categories: &[&str] = &[
        "math",
        "statistical",
        "array",
        "filter",
        "lookup",
        "text",
        "date",
        "engineering",
        "financial",
        "database",
    ];

    for &cat in categories {
        if !all && category != Some(cat) {
            continue;
        }

        let cases = match cat {
            "math" => gen_math::generate(platform),
            "statistical" => gen_statistical::generate(platform),
            "array" => gen_array_filter::generate_array(platform),
            "filter" => gen_array_filter::generate_filter(platform),
            "lookup" => gen_lookup::generate(platform),
            "text" => gen_text_date_eng_fin::generate_text(platform),
            "date" => gen_text_date_eng_fin::generate_date(platform),
            "engineering" => gen_text_date_eng_fin::generate_engineering(platform),
            "financial" => gen_text_date_eng_fin::generate_financial(platform),
            "database" => gen_database::generate(platform),
            _ => vec![],
        };

        let file_path = out_dir.join(format!("{}.tsv", cat));
        generate::write_tsv(&cases, &file_path)?;
        println!("wrote {} ({} cases) → {}", cat, cases.len(), file_path.display());
    }

    Ok(())
}
