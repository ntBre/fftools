from collections import defaultdict

import qcportal
from openff.qcsubmit.results import TorsionDriveResultCollection
from openff.toolkit import ForceField

ff = ForceField(
    "/home/brent/omsf/projects/valence-fitting"
    "/01_generate-forcefield/output/initial-force-field-openff-2.1.0.offxml"
)
ds = TorsionDriveResultCollection.parse_file(
    "/home/brent/omsf/rust/coprelos/testfiles/td.json"
)

env = defaultdict(int)
rec = defaultdict(set)
smi = defaultdict(set)
tor = defaultdict(int)
for r, mol in ds.to_records():
    d = r.specification.keywords.dihedrals[0]
    smiles = mol.to_smiles()
    labels = ff.label_molecules(mol.to_topology())[0]["ProperTorsions"]
    for e, p in labels.items():
        env[p.id] += 1
        if e == d or e == d[::-1]:
            tor[p.id] += 1
            print(p.id, e, d)
        rec[p.id].add(r.id)
        smi[p.id].add(smiles)

for pid, e in sorted(env.items(), key=lambda x: -x[1]):
    print(
        f"{pid:<6} {e:>8} {len(rec[pid]):>8} {len(smi[pid]):>8} {tor[pid]:>8}"
    )
