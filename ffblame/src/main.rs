//! read ib output CSV files and assign errors to parameters

use fftools::{
    die, load_csv, load_dataset, parameter_map::ParameterMap, Record,
};
use log::debug;
use openff_toolkit::ForceField;
use rayon::prelude::*;
use rdkit_rs::ROMol;
use std::collections::{HashMap, HashSet};

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// TODO there is a question here of whether or not to consider only unique
/// values. it might actually make more sense to count it as an additional error
/// for each occurrence of the parameter in a molecule. this only counts once
/// per record
fn process_records(
    records: Vec<Record>,
    dataset: HashMap<String, String>,
    params: ParameterMap,
) -> Vec<(String, f64)> {
    records
        .into_par_iter()
        .flat_map(|r| {
            let smiles = dataset.get(&r.id.to_string()).unwrap();
            // TODO should we be cleaning here?
            let mol = ROMol::from_smiles(smiles);
            let pids: HashSet<_> =
                params.label_molecule(&mol).into_values().collect();
            pids.into_iter()
                .zip(std::iter::repeat(r.value))
                .collect::<Vec<_>>() /* could be replaced with .par_bridge */
        })
        .collect()
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
    let res = process_records(records, dataset, params);

    let mut errors: HashMap<String, Vec<f64>> = HashMap::new();
    for (pid, val) in res {
        errors.entry(pid).or_default().push(val);
    }

    println!("param,mean");
    for (pid, errs) in errors {
        println!("{pid},{:.8}", mean(&errs));
    }
}
