//! read ib output CSV files and split it into one subset matching a group of
//! parameters and one subset not matching the same parameters

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::fs::read_to_string;
use std::io;
use std::path::Path;

use clap::Parser;
use rayon::prelude::*;

use fftools::parameter_map::ParameterMap;
use fftools::{die, load_csv, load_dataset, Record};
use openff_toolkit::ForceField;
use rdkit_rs::ROMol;

use crate::cli::Cli;

#[cfg(test)]
mod tests;

/// Load a sequence of whitespace-separated parameter IDs from `path`
fn load_subset(path: impl AsRef<Path>) -> io::Result<Vec<String>> {
    Ok(read_to_string(path)?
        .split_ascii_whitespace()
        .map(String::from)
        .collect())
}

/// Convert a sequence of [Record]s into a sequence of [Record], parameter ID
/// pairs
fn process_records(
    records: Vec<Record>,
    dataset: HashMap<String, String>,
    params: ParameterMap,
) -> Vec<(Record, HashSet<String>)> {
    let map_op = |r: Record| -> (Record, HashSet<String>) {
        let smiles = dataset.get(&r.id.to_string()).unwrap();
        let mut mol = ROMol::from_smiles(smiles);
        mol.openff_clean();
        (r, params.label_molecule(&mol).into_values().collect())
    };
    records.into_par_iter().map(map_op).collect()
}

struct Output {
    in_set: Vec<(Record, HashSet<String>)>,
    out_set: Vec<(Record, HashSet<String>)>,
}

fn inner<P, Q, R>(records: P, dataset: Q, forcefield: &str, subset: R) -> Output
where
    P: AsRef<Path> + Debug,
    Q: AsRef<Path> + Debug,
    R: AsRef<Path> + Debug,
{
    let records = load_csv(&records)
        .unwrap_or_else(|e| die!("failed to load {:?} with {}", records, e));
    let dataset = load_dataset(&dataset)
        .unwrap_or_else(|e| die!("failed to load {:?} with {}", dataset, e));
    let forcefield = ForceField::load(forcefield)
        .unwrap_or_else(|e| die!("failed to load {:?} with {}", forcefield, e));
    let params: ParameterMap = forcefield
        .get_parameter_handler("ProperTorsions")
        .unwrap()
        .into();
    let subset: HashSet<_> = load_subset(&subset)
        .unwrap_or_else(|e| die!("failed to load {:?} with {}", subset, e))
        .into_iter()
        .collect();

    let processed_records = process_records(records, dataset, params);

    let (in_set, out_set): (Vec<_>, Vec<_>) = processed_records
        .into_iter()
        .partition(|(_r, pids)| pids.intersection(&subset).count() > 0);
    Output { in_set, out_set }
}

mod cli {
    use std::path::PathBuf;

    use clap::Parser;

    #[derive(Parser)]
    #[command(version, about, long_about = None)]
    pub struct Cli {
        #[arg(short, long)]
        pub records: PathBuf,

        #[arg(short, long)]
        pub dataset: PathBuf,

        #[arg(short, long)]
        pub forcefield: String,

        #[arg(short, long)]
        pub subset: PathBuf,

        #[arg(short, long, default_value_t = 0)]
        pub threads: usize,

        #[arg(short, long)]
        pub output_base: Option<PathBuf>,
    }
}

fn main() {
    let args = Cli::parse();

    rayon::ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build_global()
        .expect("failed to initialize thread pool");

    let Output { in_set, out_set } =
        inner(args.records, args.dataset, &args.forcefield, args.subset);

    let (mut win, mut wout): (
        Box<dyn std::io::Write>,
        Box<dyn std::io::Write>,
    ) = if let Some(base) = args.output_base {
        let in_file = std::fs::File::create(base.with_extension("in")).unwrap();
        let out_file =
            std::fs::File::create(base.with_extension("out")).unwrap();
        (Box::new(in_file), Box::new(out_file))
    } else {
        (Box::new(std::io::stdout()), Box::new(std::io::stdout()))
    };

    for (rec, _) in in_set {
        writeln!(win, "inset,{},{}", rec.id, rec.value).unwrap();
    }
    for (rec, _) in out_set {
        writeln!(wout, "outset,{},{}", rec.id, rec.value).unwrap();
    }
}
