use serde::Deserialize;
use std::{collections::HashMap, fs::read_to_string, io, path::Path};

pub mod parameter_map;

pub struct Record {
    /// the QCArchive record ID
    pub id: usize,
    pub value: f64,
}

/// load a simple CSV file from `path`, skipping one header line, and returning
/// the remaining lines as a sequence of [Record]s
pub fn load_csv(path: impl AsRef<Path>) -> io::Result<Vec<Record>> {
    Ok(read_to_string(path)?
        .lines()
        .skip(1) // header
        .map(|s| {
            let sp: Vec<_> = s.split(',').map(str::trim).collect();
            assert_eq!(sp.len(), 2);
            Record {
                id: sp[0].parse().unwrap(),
                value: sp[1].parse().unwrap(),
            }
        })
        .collect())
}

#[derive(Deserialize)]
pub struct Entry {
    /// the QCArchive record ID
    pub record_id: String,

    /// the canonical SMILES string representing the molecule
    pub cmiles: String,
}

#[derive(Deserialize)]
pub struct Dataset {
    pub entries: HashMap<String, Vec<Entry>>,
}

/// load a dataset from `path` and return it as a map of record ID to SMILES
pub fn load_dataset(
    path: impl AsRef<Path>,
) -> io::Result<HashMap<String, String>> {
    let ds: Dataset = serde_json::from_str(&read_to_string(path)?)?;
    Ok(ds
        .entries
        .into_values()
        .flatten()
        .map(|rec| (rec.record_id, rec.cmiles))
        .collect())
}
