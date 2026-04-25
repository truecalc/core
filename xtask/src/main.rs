mod gen_array_filter;
mod gen_database;
mod gen_logical_info;
mod gen_lookup;
mod gen_math;
mod gen_operator;
mod gen_parser_web;
mod gen_statistical;
mod gen_text_date_eng_fin;
mod generate;
mod sheets_eval;
mod types;

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
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
#[command(
    name = "xtask",
    about = "Fixture generation and evaluation tasks"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Generate pre-evaluation fixture TSV files
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
    /// Evaluate fixtures against Google Sheets (stub — see T3.8)
    GsEvaluate {
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
        Commands::GenerateFixtures {
            platform,
            category,
            all,
        } => {
            if !all && category.is_none() {
                bail!("Specify --all or --category <name>");
            }
            run_generate_fixtures(platform.to_platform(), category.as_deref(), all)?;
        }
        Commands::GsEvaluate {
            platform,
            all,
            category,
        } => {
            if !all && category.is_none() {
                bail!("Specify --all or --category <name>");
            }
            let web_app_url = std::env::var("GAS_URL")
                .context("GAS_URL env var not set (set it to the Apps Script web app URL)")?;
            run_gs_evaluate(platform.to_platform(), category.as_deref(), all, &web_app_url)?;
        }
    }

    Ok(())
}

fn read_tsv(path: &std::path::Path) -> Result<Vec<types::TestCase>> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(path)
        .with_context(|| format!("open TSV: {}", path.display()))?;

    let mut cases = Vec::new();
    for result in rdr.records() {
        let record = result?;
        // Columns: description, formula_text, expected_value, test_category, expected_type
        let description = record.get(0).unwrap_or("").to_string();
        let formula_text = record.get(1).unwrap_or("").to_string();
        let expected_value = record.get(2).unwrap_or("").to_string();
        let test_category = record.get(3).unwrap_or("").to_string();
        let expected_type = record.get(4).unwrap_or("").to_string();
        cases.push(types::TestCase {
            description,
            formula: formula_text,
            expected_value,
            test_category,
            expected_type,
        });
    }
    Ok(cases)
}

fn run_gs_evaluate(
    platform: Platform,
    category: Option<&str>,
    all: bool,
    web_app_url: &str,
) -> Result<()> {
    let input_dir = PathBuf::from("target/fixture-gen").join(platform.dir_name());
    let output_dir = PathBuf::from("crates/core/tests/fixtures").join(platform.dir_name());
    std::fs::create_dir_all(&output_dir)?;

    let evaluator = sheets_eval::SheetsEvaluator::new(web_app_url.to_owned());

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
        "logical",
        "info",
        "operator",
        "parser",
        "web",
    ];

    for &cat in categories {
        if !all && category != Some(cat) {
            continue;
        }

        let input_path = input_dir.join(format!("{}.tsv", cat));
        if !input_path.exists() {
            println!("skipping {} — input TSV not found: {}", cat, input_path.display());
            continue;
        }

        println!("evaluating {} …", cat);
        let cases = read_tsv(&input_path)?;
        let evaluated = evaluator.evaluate(&cases)?;

        // write_tsv writes "={formula}" — but our TestCase.formula field already
        // has the "=" prefix from the input TSV, so strip it before passing in.
        let stripped: Vec<types::TestCase> = evaluated
            .into_iter()
            .map(|mut c| {
                if c.formula.starts_with('=') {
                    c.formula = c.formula[1..].to_string();
                }
                c
            })
            .collect();

        let output_path = output_dir.join(format!("{}.tsv", cat));
        generate::write_tsv(&stripped, &output_path)?;
        println!(
            "wrote {} ({} cases) → {}",
            cat,
            stripped.len(),
            output_path.display()
        );
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
        "logical",
        "info",
        "operator",
        "parser",
        "web",
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
            "logical" => gen_logical_info::generate_logical(platform),
            "info" => gen_logical_info::generate_info(platform),
            "operator" => gen_operator::generate_operator(platform),
            "parser" => gen_parser_web::generate_parser(platform),
            "web" => gen_parser_web::generate_web(platform),
            _ => vec![],
        };

        let file_path = out_dir.join(format!("{}.tsv", cat));
        generate::write_tsv(&cases, &file_path)?;
        println!(
            "wrote {} ({} cases) → {}",
            cat,
            cases.len(),
            file_path.display()
        );
    }

    Ok(())
}
