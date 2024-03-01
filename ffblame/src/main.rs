//! read ib output CSV files and assign errors to parameters

use serde::Deserialize;
use std::{collections::HashMap, fs::read_to_string, io, path::Path};

macro_rules! die {
    ($($t:tt)*) => {{
        eprintln!($($t)*);
        std::process::exit(1);
    }};
}

struct Record {
    id: usize,
    value: f64,
}

fn load_csv(path: impl AsRef<Path>) -> io::Result<Vec<Record>> {
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
struct Entry {
    record_id: String,
    cmiles: String,
}

#[derive(Deserialize)]
struct Dataset {
    entries: HashMap<String, Vec<Entry>>,
}

fn load_dataset(path: impl AsRef<Path>) -> io::Result<HashMap<String, String>> {
    let ds: Dataset = serde_json::from_str(&read_to_string(path)?)?;
    Ok(ds
        .entries
        .into_values()
        .flatten()
        .map(|rec| (rec.record_id, rec.cmiles))
        .collect())
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 {
        die!("Usage: ffblame <data.csv> <dataset.json>");
    }
    let records = load_csv(&args[1])
        .unwrap_or_else(|e| die!("failed to load {} with {}", args[1], e));
    let dataset = load_dataset(&args[2])
        .unwrap_or_else(|e| die!("failed to load {} with {}", args[2], e));
    println!(
        "loaded {} records, {} in dataset",
        records.len(),
        dataset.len()
    );
}
