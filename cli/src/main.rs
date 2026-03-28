mod rules;

use clap::{Parser, Subcommand};
use rules::{generate_html, load_categories, load_lookup, prompt_for_category};
use std::fs;

use crate::rules::{CsvRecord, find_category};

#[derive(Parser)]
#[command(name = "auseinnahmen")]
#[command(about = "Categorize transactions from banking CSV exports")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Process CSV and generate category rules
    Rules {
        /// Path to banking CSV export
        csv: String,

        /// Path to category definitions JSON
        categories: String,

        /// Path to existing category lookup JSON (optional)
        #[arg(short, long)]
        lookup: Option<String>,

        /// Path to write output JSON (optional, defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,

        /// Override lookup file with output
        #[arg(short = 'i', long = "in-place")]
        in_place: bool,

        /// Bank type (gls or dkb)
        #[arg(short, long, default_value = "gls")]
        bank: String,
    },

    /// Transform CSV with categories and lookup to HTML
    Transform {
        /// Path to banking CSV export
        csv: String,

        /// Path to category definitions JSON
        categories: String,

        /// Path to category lookup JSON
        lookup: String,

        /// Path to write output HTML (optional, defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,

        /// Bank type (gls or dkb)
        #[arg(short, long, default_value = "gls")]
        bank: String,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Rules {
            csv,
            categories,
            lookup: lookup_arg,
            output,
            in_place: override_lookup,
            bank,
        } => {
            run_rules(
                &csv,
                &categories,
                lookup_arg.as_deref(),
                output.as_deref(),
                override_lookup,
                &bank,
            );
        }
        Commands::Transform {
            csv,
            categories,
            lookup,
            output,
            bank,
        } => {
            run_transform(&csv, &categories, &lookup, output.as_deref(), &bank);
        }
    }
}

fn run_rules(
    csv: &str,
    categories: &str,
    lookup_arg: Option<&str>,
    output: Option<&str>,
    override_lookup: bool,
    bank: &str,
) {
    let categories = load_categories(categories).expect("Failed to load categories");
    let mut lookup = match lookup_arg {
        Some(path) => load_lookup(path).expect("Failed to parse previous rules."),
        None => {
            if override_lookup {
                eprintln!("Error: --override-lookup requires --lookup to be specified");
                std::process::exit(1);
            }
            Vec::new()
        }
    };

    let records = if bank == "dkb" {
        rules::read_dkb_csv(csv).expect("Failed to read DKB CSV")
    } else {
        rules::read_gls_csv(csv).expect("Failed to read CSV")
    };

    eprintln!("# categories");
    for category in &categories {
        eprintln!(
            "## [{}] {}\n{}",
            category.index, category.title, category.description
        );
    }
    for record in &records {
        if !find_category(record, &lookup)
            && let Some(category) = prompt_for_category(&categories, record)
        {
            lookup.push(category);
        }
    }

    let json = serde_json::to_string_pretty(&lookup).unwrap();

    if override_lookup {
        fs::write(lookup_arg.unwrap(), &json).expect("Failed to write output file");
    } else if let Some(path) = output {
        fs::write(path, &json).expect("Failed to write output file");
    } else {
        println!("{}", json);
    }
}

fn run_transform(csv: &str, categories: &str, lookup: &str, output: Option<&str>, bank: &str) {
    let categories = load_categories(categories).expect("Failed to load categories");
    let lookup = load_lookup(lookup).expect("Failed to load lookup");

    let records = if bank == "dkb" {
        rules::read_dkb_csv(csv).expect("Failed to read DKB CSV")
    } else {
        rules::read_gls_csv(csv).expect("Failed to read CSV")
    };

    let filtered_records: Vec<CsvRecord> = records
        .into_iter()
        .filter(|r| r.amount.replace(',', "").starts_with('-'))
        .collect();

    let html = generate_html(&filtered_records, &lookup, &categories);

    if let Some(path) = output {
        fs::write(path, &html).expect("Failed to write output file");
    } else {
        println!("{}", html);
    }
}
