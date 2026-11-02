use tor_consdiff::DiffSizeStrictness::None as NoSizeCheck;

#[test]
fn apply_tricky() {
    let pre = include_str!("../testdata/consensus1.txt");
    let diff = include_str!("../testdata/diff1.txt");
    let post = include_str!("../testdata/consensus2.txt");

    let result = tor_consdiff::apply_diff(pre, diff, None, NoSizeCheck).unwrap();
    assert_eq!(result.to_string(), post);
}

#[test]
fn apply_replacement() {
    let base = include_str!("../testdata/consensus1.txt");
    let target = include_str!("../testdata/consensus2.txt");
    // Use the independently generated fixture's frozen signed-input and
    // target hashes, then replace the whole document without an LCS provider.
    let header = include_str!("../testdata/diff1.txt")
        .lines()
        .take(2)
        .collect::<Vec<_>>()
        .join("\n");
    let diff = format!("{header}\n1,$d\n0a\n{target}.\n");

    assert_eq!(
        diff.lines().take(2).collect::<Vec<_>>(),
        [
            "network-status-diff-version 1",
            "hash B03DA3ACA1D3C1D083E3FF97873002416EBD81A058B406D5C5946EAB53A79663 F6789F35B6B3BA58BB23D29E53A8ED6CBB995543DBE075DD5671481C4BA677FB"
        ]
    );

    let expected_input =
        hex::decode("B03DA3ACA1D3C1D083E3FF97873002416EBD81A058B406D5C5946EAB53A79663")
            .unwrap()
            .try_into()
            .unwrap();
    let result = tor_consdiff::apply_diff(base, &diff, Some(expected_input), NoSizeCheck).unwrap();
    result.check_digest().unwrap();
    assert_eq!(result.to_string(), target);
    assert!(tor_consdiff::apply_diff(base, &diff, Some([0; 32]), NoSizeCheck).is_err());
}
