use social_deduction::{experiment007 as e7, experiment012::*};
use std::{fs, path::Path};

#[test]
fn frozen_complete_grid_raw_bytes_and_actual_windows_manifest_reproduce() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let raw = fs::read(root.join("evidence/experiment-012.json")).unwrap();
    assert_eq!(raw.len(), 300005474);
    assert_eq!(
        e7::sha256(&raw),
        "f22a61dbeca029dd29977228805ab9c932f5e834433c61c8edaca1da0157b4f2"
    );
    let expected: Evaluation = serde_json::from_slice(&raw).unwrap();
    assert_eq!(
        (
            expected.rows.len(),
            expected.incidents,
            expected.replay_matches,
            expected.independent_worlds,
            expected.comparisons.len(),
            expected.prediction_groups.len()
        ),
        (696, 5568, 696, 64, 66, 1728)
    );
    let actual = evaluate().unwrap();
    assert!(actual == expected, "complete frozen 012 values differ");
    drop(expected);
    assert!(
        serde_json::to_vec(&actual).unwrap() == raw,
        "frozen raw 012 bytes differ"
    );
    drop(raw);
    let platform_raw = fs::read(root.join("evidence/experiment-012-windows.json")).unwrap();
    assert_eq!(platform_raw.len(), 466173);
    assert_eq!(
        e7::sha256(&platform_raw),
        "de9aebfcf050b92e3a518c31c8139cc54ec5da894d8f65b546c6254a26d7e1c2"
    );
    let windows: e7::PlatformReplay = serde_json::from_slice(&platform_raw).unwrap();
    assert_eq!(windows.execution.os, "windows");
    assert_eq!(windows.execution.architecture, "x86_64");
    assert_eq!(
        windows.execution.revision,
        "9cdbf593dc7361108939e0e6954a024b6b6a0e9d"
    );
    assert_eq!(
        windows.execution.lock_digest,
        e7::sha256(&fs::read(root.join("Cargo.lock")).unwrap())
    );
    assert_eq!(windows.manifest, manifest(&actual).unwrap());
    assert_eq!(
        windows.manifest.evaluation_digest,
        "7f59152e77d500b07b038d4551aa4f0fc69bf241a2edbfb72d5c1d614c905e9b"
    );
    assert_eq!(
        windows.manifest.metric_digest,
        "323b83810219245c3aad5d33a0f21befcd74a931454f59dd36380a4010e44a6d"
    );
    assert!(
        e7::compare_platforms(&windows, &windows).is_err(),
        "two Windows executions cannot prove Linux agreement"
    );
}
