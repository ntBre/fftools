use log::trace;
use openff_toolkit::typing::engines::smirnoff::parameters::ParameterHandler;
use rdkit_rs::{find_smarts_matches_mol, ROMol};

use std::collections::HashMap;

use crate::Pid;

/// A [ParameterMap] is basically the Rust/RDKit-compatible version of an OpenFF
/// ForceField. Instead of having to access parameter SMIRKS patterns as Strings
/// and pass them to RDKit as such, a [ParameterMap] converts them all to
/// [ROMol] up front for faster matching in [ParameterMap::label_molecule]. TODO
/// support more than one [ParameterHandler] at a time.
pub struct ParameterMap(Vec<(Pid, ROMol)>);

impl ParameterMap {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn keys(&self) -> impl Iterator<Item = &Pid> {
        self.0.iter().map(|(pid, _mol)| pid)
    }

    /// label `mol` with `params` and return a map of chemical environment
    /// tuples to parameter IDs
    pub fn label_molecule(&self, mol: &ROMol) -> HashMap<Vec<usize>, String> {
        let mut matches = HashMap::new();
        for (id, smirks) in &self.0 {
            let env_matches = find_smarts_matches_mol(mol, smirks);
            for mut mat in env_matches {
                if mat.first().unwrap() > mat.last().unwrap() {
                    mat.reverse();
                }
                trace!("{mat:?} => {id}");
                matches.insert(mat, id.clone());
            }
        }
        matches
    }
}

impl From<ParameterHandler> for ParameterMap {
    fn from(ph: ParameterHandler) -> Self {
        Self(
            ph.parameters()
                .into_iter()
                .map(|p| (p.id(), ROMol::from_smarts(&p.smirks())))
                .collect(),
        )
    }
}
