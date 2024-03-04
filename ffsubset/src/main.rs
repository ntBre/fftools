//! read ib output CSV files and split it into one subset matching a group of
//! parameters and one subset not matching the same parameters

use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;
use std::io;
use std::path::Path;

use rayon::prelude::*;

use fftools::parameter_map::ParameterMap;
use fftools::{load_csv, load_dataset, Record};
use openff_toolkit::ForceField;
use rdkit_rs::ROMol;

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
        // TODO should we be cleaning here?
        let mol = ROMol::from_smiles(smiles);
        (r, params.label_molecule(&mol).into_values().collect())
    };
    records.into_par_iter().map(map_op).collect()
}

fn main() {
    let args = [
        "ffsubset",
        "testfiles/dde.csv",
        "testfiles/industry.json",
        "openff-2.1.0.offxml",
        "testfiles/subset.in",
    ];
    let records = load_csv(&args[1]).unwrap();
    let dataset = load_dataset(&args[2]).unwrap();
    let forcefield = ForceField::load(&args[3]).unwrap();
    let params: ParameterMap = forcefield
        .get_parameter_handler("ProperTorsions")
        .unwrap()
        .into();
    let subset: HashSet<_> =
        load_subset(&args[4]).unwrap().into_iter().collect();

    let processed_records = process_records(records, dataset, params);

    let (int, out): (Vec<_>, Vec<_>) = processed_records
        .into_iter()
        .partition(|(_r, pids)| pids.intersection(&subset).count() > 1);

    dbg!(int.len(), out.len());
}
