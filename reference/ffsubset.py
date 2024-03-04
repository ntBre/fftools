import logging

import pandas as pd
from openff.qcsubmit.results import OptimizationResultCollection
from openff.toolkit import ForceField, Molecule
from tqdm import tqdm

logging.getLogger("openff").setLevel(logging.ERROR)


def load_csv(filename):
    dde = pd.read_csv(filename)
    dde.columns = ["record", "diff"]
    return {str(r): d for r, d in zip(dde["record"], dde["diff"])}


def load_dataset(filename):
    ds = OptimizationResultCollection.parse_file(filename)
    return {
        v.record_id: v.cmiles for value in ds.entries.values() for v in value
    }


def load_subset(filename):
    with open(filename) as inp:
        return {line.strip() for line in inp}


args = [
    "ffsubset",
    "testfiles/dde.csv",
    "testfiles/industry.json",
    "openff-2.1.0.offxml",
    "testfiles/subset.in",
]

records = load_csv(args[1])
dataset = load_dataset(args[2])
forcefield = ForceField(args[3])
subset = load_subset(args[4])

processed_records = []
for rec_id, _val in tqdm(records.items()):
    smiles = dataset[rec_id]
    mol = Molecule.from_mapped_smiles(smiles, allow_undefined_stereo=True)
    labels = {
        r.id
        for r in forcefield.label_molecules(mol.to_topology())[0][
            "ProperTorsions"
        ].values()
    }
    processed_records.append((rec_id, labels))

in_set, out_set = 0, 0
for _rec_id, pids in processed_records:
    if len(pids.intersection(subset)) > 0:
        in_set += 1
    else:
        out_set += 1

print(in_set, out_set)

# => 58825 12935
