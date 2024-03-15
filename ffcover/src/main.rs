//! compute dataset coverage for a force field

use openff_qcsubmit::results::BaseResultCollection;
use openff_qcsubmit::results::TorsionDriveResultCollection;

use std::path::Path;
use std::{
    cmp,
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

    #[arg(short, long, default_value_t = false)]
    torsions: bool,
}

#[derive(Default)]
struct Match {
    env: usize,
    rec: HashSet<String>,
    mol: HashSet<Smiles>,
    tor: usize,
}

impl Match {
    fn by_env(&self, other: &Self) -> cmp::Ordering {
        self.env.cmp(&other.env)
    }
}

/// Process a dataset using only the data in the dataset, without contacting
/// QCArchive to retrieve record information
fn opt_main(dataset: HashMap<String, String>, params: ParameterMap) {
    let mut matches: HashMap<Pid, Match> = params
        .keys()
        .cloned()
        .map(|pid| (pid, Match::default()))
        .collect();
    for (rec_id, smiles) in dataset {
        let mut mol = ROMol::from_smiles(&smiles);
        mol.openff_clean();
        let labels = params.label_molecule(&mol);
        for (_env, id) in labels {
            // pid must have been pre-inserted into the map
            let entry = matches.get_mut(&id).unwrap();
            entry.env += 1;
            entry.rec.insert(rec_id.clone());
            entry.mol.insert(smiles.clone());
        }
    }

    let mut matches: Vec<_> = matches.into_iter().collect();
    matches.sort_by(|(_, a), (_, b)| Match::by_env(b, a));
    // rev by env count

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
    for (pid, Match { env, rec, mol, .. }) in matches {
        let rec = rec.len();
        let smi = mol.len();
        println!("{pid:<pid_w$} {env:>env_w$} {rec:>rec_w$} {smi:>smi_w$}");
    }
}

/// Process a dataset after contacting QCArchive to retrieve the TorsionDrive
/// record information
fn td_main(dataset: impl AsRef<Path>, ff: ForceField) {
    let mut matches: HashMap<Pid, Match> = ff
        .get_parameter_handler("ProperTorsions")
        .unwrap()
        .parameters()
        .into_iter()
        .map(|p| (p.id(), Match::default()))
        .collect();
    let ds = TorsionDriveResultCollection::parse_file(dataset).unwrap();
    for (rec, mol) in ds.to_records() {
        let d = &rec.specification.keywords.dihedrals[0];
        // this is definitely 0-indexed, at least it sometimes has zeros
        let dihedral = vec![d.0, d.1, d.2, d.3];
        let labels = ff.label_molecules(mol.to_topology())[0]
            .remove("ProperTorsions")
            .unwrap();
        let smiles = mol.to_smiles_default();
        for (mut env, p) in labels {
            // pid must have been pre-inserted into the map
            let entry = matches.get_mut(&p.id()).unwrap();
            entry.env += 1;
            entry.rec.insert(rec.id.to_string());
            entry.mol.insert(smiles.clone());
            if env == dihedral || {
                env.reverse();
                env == dihedral
            } {
                entry.tor += 1;
            }
        }
    }
    let mut matches: Vec<_> = matches.into_iter().collect();
    matches.sort_by(|(_, a), (_, b)| Match::by_env(b, a)); // rev by env count

    let pid_w = 6;
    let env_w = 8;
    let rec_w = 8;
    let smi_w = 8;
    let tor_w = 8;
    println!(
        "{pid:<pid_w$} {env:>env_w$} {rec:>rec_w$} {smi:>smi_w$} {tor:>tor_w$}",
        pid = "pid",
        env = "env",
        rec = "rec",
        smi = "smi",
        tor = "tor",
    );
    for (pid, Match { env, rec, mol, tor }) in matches {
        let rec = rec.len();
        let smi = mol.len();
        println!(
            "{pid:<pid_w$} {env:>env_w$} {rec:>rec_w$} \
             {smi:>smi_w$} {tor:>tor_w$}"
        );
    }
}

fn main() {
    let cli = Cli::parse();
    let ff = ForceField::load(&cli.forcefield).unwrap();

    if cli.torsions {
        td_main(&cli.dataset, ff);
    } else {
        let params: ParameterMap =
            ff.get_parameter_handler("ProperTorsions").unwrap().into();
        let dataset = load_dataset(&cli.dataset).unwrap();
        opt_main(dataset, params);
    }
}
