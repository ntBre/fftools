use openff_toolkit::ForceField;

fn get_params(ff: &str) -> Vec<(String, f64)> {
    let h = ForceField::load(ff)
        .unwrap()
        .get_parameter_handler("ProperTorsions")
        .unwrap();
    let mut ret = Vec::new();
    for p in h.parameters() {
        for (i, k) in p.k().into_iter().enumerate() {
            ret.push((p.id() + "." + &(i + 1).to_string(), k));
        }
    }
    ret
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 {
        fftools::die!("Usage: ffdiff <ff1.offxml> <ff2.offxml>...");
    }
    let p1 = get_params(&args[1]);
    let mut ps = Vec::new();
    for arg in &args[2..] {
        ps.push(get_params(&arg));
    }

    print!("param");
    for a in &args[1..] {
        print!(" {}", a.strip_suffix(".offxml").unwrap());
    }
    println!();

    for (k, v) in p1 {
        print!("{k} {v}");
        for p2 in &ps {
            if let Some((_, v2)) = p2.iter().find(|(n, _)| n == &k) {
                print!(" {v2}");
            } else {
                print!(" NA");
            }
        }
        println!();
    }
}
