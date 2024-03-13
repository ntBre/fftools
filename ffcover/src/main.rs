//! compute dataset coverage for a force field

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use clap::Parser;
use fftools::{load_dataset, parameter_map::ParameterMap, Pid};
use openff_toolkit::ForceField;
use rdkit_rs::ROMol;

const DS: &str = "/home/brent/omsf/projects/valence-fitting/\
                    02_curate-data/sage/filtered-opt.json";

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = "openff-2.1.0.offxml")]
    forcefield: String,

    #[arg(short, long, default_value = DS)]
    dataset: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    let ff = ForceField::load(&cli.forcefield).unwrap();
    let params: ParameterMap =
        ff.get_parameter_handler("ProperTorsions").unwrap().into();
    let dataset = load_dataset(&cli.dataset).unwrap();

    let mut env_matches: HashMap<Pid, usize> = HashMap::new();
    let mut rec_matches: HashMap<Pid, HashSet<String>> = HashMap::new();
    for (rec_id, smiles) in dataset {
        let mut mol = ROMol::from_smiles(&smiles);
        mol.openff_clean();
        let labels = params.label_molecule(&mol);
        for (_env, id) in labels {
            *env_matches.entry(id.clone()).or_default() += 1;
            rec_matches
                .entry(id.clone())
                .or_default()
                .insert(rec_id.clone());
        }
    }

    let mut env_matches: Vec<_> = env_matches.into_iter().collect();
    env_matches.sort_by(|a, b| b.1.cmp(&a.1)); // reversed by count

    let pid_w = 6;
    let env_w = 8;
    let rec_w = 8;
    println!(
        "{pid:<pid_w$} {env:>env_w$} {rec:>rec_w$}",
        pid = "pid",
        env = "env",
        rec = "rec"
    );
    for (pid, count) in env_matches {
        let rec = if let Some(s) = rec_matches.get(&pid) {
            s.len()
        } else {
            0
        };
        println!("{pid:<pid_w$} {count:>env_w$} {rec:>rec_w$}");
    }
}
