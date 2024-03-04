//! read ib output CSV files and split it into one subset matching a group of
//! parameters and one subset not matching the same parameters

use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;
use std::io;
use std::path::Path;

use rayon::prelude::*;

use fftools::parameter_map::ParameterMap;
use fftools::{die, load_csv, load_dataset, Record};
use openff_toolkit::ForceField;
use rdkit_rs::ROMol;

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

#[allow(clippy::needless_borrow, clippy::needless_borrows_for_generic_args)]
fn inner(args: &[&str]) -> Output {
    assert_eq!(args.len(), 5);
    let records = load_csv(&args[1])
        .unwrap_or_else(|e| die!("failed to load {} with {}", args[1], e));
    let dataset = load_dataset(&args[2]).unwrap();
    let forcefield = ForceField::load(&args[3]).unwrap();
    let params: ParameterMap = forcefield
        .get_parameter_handler("ProperTorsions")
        .unwrap()
        .into();
    let subset: HashSet<_> =
        load_subset(&args[4]).unwrap().into_iter().collect();

    let processed_records = process_records(records, dataset, params);

    let (in_set, out_set): (Vec<_>, Vec<_>) = processed_records
        .into_iter()
        .partition(|(_r, pids)| pids.intersection(&subset).count() > 0);
    Output { in_set, out_set }
}

fn main() {
    let args = [
        "ffsubset",
        "testfiles/dde.csv",
        "testfiles/industry.json",
        "openff-2.1.0.offxml",
        "testfiles/subset.in",
    ];
    let res = inner(&args);

    dbg!(res.in_set.len(), res.out_set.len());
}
