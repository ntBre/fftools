//! read ib output CSV files and assign errors to parameters

use fftools::{label_molecule, ParameterMap};
use log::debug;
use openff_toolkit::ForceField;
use rayon::prelude::*;
use rdkit_rs::ROMol;
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    fs::read_to_string,
    io,
    path::Path,
};

macro_rules! die {
    ($($t:tt)*) => {{
        eprintln!($($t)*);
        std::process::exit(1);
    }};
}

struct Record {
    /// the QCArchive record ID
    id: usize,
    value: f64,
}

/// load a simple CSV file from `path`, skipping one header line, and returning
/// the remaining lines as a sequence of [Record]s
fn load_csv(path: impl AsRef<Path>) -> io::Result<Vec<Record>> {
    Ok(read_to_string(path)?
        .lines()
        .skip(1) // header
        .map(|s| {
            let sp: Vec<_> = s.split(',').map(str::trim).collect();
            assert_eq!(sp.len(), 2);
            Record {
                id: sp[0].parse().unwrap(),
                value: sp[1].parse().unwrap(),
            }
        })
        .collect())
}

#[derive(Deserialize)]
struct Entry {
    /// the QCArchive record ID
    record_id: String,

    /// the canonical SMILES string representing the molecule
    cmiles: String,
}

#[derive(Deserialize)]
struct Dataset {
    entries: HashMap<String, Vec<Entry>>,
}

/// load a dataset from `path` and return it as a map of record ID to SMILES
fn load_dataset(path: impl AsRef<Path>) -> io::Result<HashMap<String, String>> {
    let ds: Dataset = serde_json::from_str(&read_to_string(path)?)?;
    Ok(ds
        .entries
        .into_values()
        .flatten()
        .map(|rec| (rec.record_id, rec.cmiles))
        .collect())
}

fn mean(v: &Vec<f64>) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

fn main() {
    env_logger::init();

    let args: Vec<_> = std::env::args().collect();
    if args.len() < 4 {
        die!("Usage: ffblame <data.csv> <dataset.json> <forcefield.offxml>");
    }
    debug!("loading CSV from {}", &args[1]);
    let records = load_csv(&args[1])
        .unwrap_or_else(|e| die!("failed to load {} with {}", args[1], e));
    debug!("loading dataset from {}", &args[2]);
    let dataset = load_dataset(&args[2])
        .unwrap_or_else(|e| die!("failed to load {} with {}", args[2], e));
    debug!("loading forcefield from {}", &args[3]);
    let forcefield = ForceField::load(&args[3])
        .unwrap_or_else(|e| die!("failed to load {} with {}", args[3], e));
    debug!("building parameter smirks");
    let params: ParameterMap = forcefield
        .get_parameter_handler("ProperTorsions")
        .unwrap()
        .into();

    debug!(
        "loaded {} records, {} in dataset, {} proper torsions",
        records.len(),
        dataset.len(),
        params.len(),
    );

    debug!("processing records");
    let res: Vec<_> = records
        .into_par_iter()
        .flat_map(|r| {
            let smiles = dataset.get(&r.id.to_string()).unwrap();
            let mol = ROMol::from_smiles(&smiles);
            // question here of whether or not to consider only unique values.
            // it might actually make more sense to count it as an additional
            // error for each occurrence of the parameter in a molecule. this
            // only counts once per record
            let pids: HashSet<_> =
                label_molecule(&mol, &params).into_values().collect();
            let vals = vec![r.value; pids.len()];
            pids.into_iter().zip(vals).collect::<Vec<_>>()
        })
        .collect();

    let mut errors: HashMap<String, Vec<f64>> = HashMap::new();
    for (pid, val) in res {
        errors.entry(pid).or_default().push(val);
    }

    println!("param,mean");
    for (pid, errs) in errors {
        println!("{pid},{:.8}", mean(&errs));
    }
}
