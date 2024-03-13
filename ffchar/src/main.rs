//! characterize the improvements and degradations for specific records with the
//! goal of identifying structural commonalities

use fftools::{
    die, load_csv, load_dataset, parameter_map::ParameterMap, Record,
};
use openff_toolkit::ForceField;
use rayon::prelude::*;
use rdkit_rs::ROMol;
use std::collections::HashMap;

fn _mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

#[allow(unused)]
struct MRecord {
    /// record_id
    id: usize,
    /// value from CSV
    value: f64,
    smiles: String,
    mol: ROMol,
    /// associated parameter IDs
    pids: Vec<String>,
}

fn process_records(
    records: Vec<Record>,
    dataset: HashMap<String, String>,
    params: ParameterMap,
) -> Vec<MRecord> {
    let map_op = |r: Record| -> MRecord {
        let smiles = dataset.get(&r.id.to_string()).unwrap();
        let mut mol = ROMol::from_smiles(smiles);
        mol.openff_clean();
        let mut pids: Vec<_> =
            params.label_molecule(&mol).into_values().collect();
        pids.sort();
        pids.dedup();
        MRecord {
            id: r.id,
            value: r.value,
            smiles: smiles.clone(),
            mol,
            pids,
        }
    };
    records.into_par_iter().map(map_op).collect()
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 4 {
        die!("Usage: ffblame <data.csv> <dataset.json> <forcefield.offxml>");
    }
    let records = load_csv(&args[1]).unwrap();
    let dataset = load_dataset(&args[2]).unwrap();
    let forcefield = ForceField::load(&args[3]).unwrap();
    let params: ParameterMap = forcefield
        .get_parameter_handler("ProperTorsions")
        .unwrap()
        .into();

    let _res = process_records(records, dataset, params);

    // let mut errors: HashMap<String, Vec<f64>> = HashMap::new();
    // for (pid, val) in res {
    //     errors.entry(pid).or_default().push(val);
    // }

    // println!("param,mean");
    // for (pid, errs) in errors {
    //     println!("{pid},{:.8}", mean(&errs));
    // }
}
