//! compute dataset coverage for a force field

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use clap::Parser;
use fftools::{load_dataset, parameter_map::ParameterMap, Pid, Smiles};
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

#[derive(Default)]
struct Match {
    env: usize,
    rec: HashSet<String>,
    mol: HashSet<Smiles>,
}

fn main() {
    let cli = Cli::parse();
    let ff = ForceField::load(&cli.forcefield).unwrap();
    let params: ParameterMap =
        ff.get_parameter_handler("ProperTorsions").unwrap().into();
    let dataset = load_dataset(&cli.dataset).unwrap();

    let mut matches: HashMap<Pid, Match> = HashMap::new();
    for (rec_id, smiles) in dataset {
        let mut mol = ROMol::from_smiles(&smiles);
        mol.openff_clean();
        let labels = params.label_molecule(&mol);
        for (_env, id) in labels {
            let entry = matches.entry(id.clone()).or_default();
            entry.env += 1;
            entry.rec.insert(rec_id.clone());
            entry.mol.insert(smiles.clone());
        }
    }

    let mut matches: Vec<_> = matches.into_iter().collect();
    matches.sort_by(|a, b| b.1.env.cmp(&a.1.env)); // reversed by env count

    let pid_w = 6;
    let env_w = 8;
    let rec_w = 8;
    let smi_w = 8;
    println!(
        "{pid:<pid_w$} {env:>env_w$} {rec:>rec_w$} {smi:>smi_w$}",
        pid = "pid",
        env = "env",
        rec = "rec",
        smi = "smi",
    );
    for (pid, Match { env, rec, mol }) in matches {
        let rec = rec.len();
        let smi = mol.len();
        println!("{pid:<pid_w$} {env:>env_w$} {rec:>rec_w$} {smi:>smi_w$}");
    }
}
