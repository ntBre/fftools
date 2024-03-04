use super::*;

#[test]
fn test_inner() {
    let Output { in_set, out_set } = inner(&[
        "ffsubset",
        "../testfiles/dde.csv",
        "../testfiles/industry.json",
        "openff-2.1.0.offxml",
        "../testfiles/subset.in",
    ]);

    assert_eq!(in_set.len(), 58825);
    assert_eq!(out_set.len(), 12935);
}
