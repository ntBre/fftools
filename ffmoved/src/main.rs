//! Track changes in parameter assignment between force fields

use fftools::{die, load_dataset, parameter_map::ParameterMap};
use openff_toolkit::ForceField;
use rayon::iter::{
    IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};
use rdkit_rs::ROMol;

fn main() {
    let args = [
        "ffmoved",
        "testfiles/industry.json",
        "openff-2.0.0.offxml",
        "openff-2.1.0.offxml",
    ];

    // usage:
    // ffmoved dataset forcefields...

    // assign parameters for each record for each force field, then see where
    // they went. going to be similar to ffblame I think with a dataset and
    // force field, but we don't need a benchmarking csv

    let dataset = load_dataset(&args[1])
        .unwrap_or_else(|e| die!("failed to load {} with {}", args[1], e));

    let molecules: Vec<(String, ROMol)> = dataset
        .into_par_iter()
        .map(|(id, smiles)| {
            let mut mol = ROMol::from_smiles(&smiles);
            mol.openff_clean();
            (id, mol)
        })
        .collect();

    let tors = "ProperTorsions";

    let p1: ParameterMap = ForceField::load(&args[2])
        .unwrap()
        .get_parameter_handler(tors)
        .unwrap()
        .into();

    let p2: ParameterMap = ForceField::load(&args[3])
        .unwrap()
        .get_parameter_handler(tors)
        .unwrap()
        .into();

    let _: Vec<()> = molecules
        .par_iter()
        .map(|(_id, mol)| {
            let l1 = p1.label_molecule(mol);
            let l2 = p2.label_molecule(mol);
            #[cfg(debug_assertions)]
            {
                // check that the two sets of chemical environments are the
                // same, otherwise the comparison is wrong and/or meaningless
                let mut k1: Vec<_> = l1.keys().collect();
                let mut k2: Vec<_> = l2.keys().collect();
                k1.sort();
                k2.sort();
                assert_eq!(k1, k2);
            }
        })
        .collect();
}
