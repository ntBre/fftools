//! read ib output CSV files and assign errors to parameters

use fftools::{label_molecule, load_csv, load_dataset, parameter_map::ParameterMap};
use log::debug;
use openff_toolkit::ForceField;
use rayon::prelude::*;
use rdkit_rs::ROMol;
use std::collections::{HashMap, HashSet};

macro_rules! die {
    ($($t:tt)*) => {{
        eprintln!($($t)*);
        std::process::exit(1);
    }};
}

fn mean(v: &[f64]) -> f64 {
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
            let mol = ROMol::from_smiles(smiles);
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
