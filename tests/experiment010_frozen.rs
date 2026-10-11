use social_deduction::{experiment007 as e7, experiment010::*};
use std::{fs, path::Path};

#[test]
fn frozen_complete_grid_raw_bytes_and_actual_windows_manifest_reproduce() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let raw = fs::read(root.join("evidence/experiment-010.json")).unwrap();
    assert_eq!(raw.len(), 972371932);
    assert_eq!(
        e7::sha256(&raw),
        "68d25a4799f79d21aff002da6867132fc1a0d69975542e3a5bc5f7d9df0905b6"
    );
    let expected: Evaluation = serde_json::from_slice(&raw).unwrap();
    assert_eq!(
        (
            expected.rows.len(),
            expected.incidents,
            expected.replay_matches,
            expected.independent_worlds
        ),
        (3632, 28864, 3632, 128)
    );
    let actual = evaluate().unwrap();
    assert!(actual == expected, "complete frozen 010 values differ");
    drop(expected);
    assert!(
        serde_json::to_vec(&actual).unwrap() == raw,
        "frozen raw 010 bytes differ"
    );
    drop(raw);
    let platform_raw = fs::read(root.join("evidence/experiment-010-windows.json")).unwrap();
    assert_eq!(platform_raw.len(), 2270551);
    assert_eq!(
        e7::sha256(&platform_raw),
        "81d712affd6dd39346e0af475c8dfdbcf7cce4920e62dbf4f856eadb3dbebed7"
    );
    let windows: e7::PlatformReplay = serde_json::from_slice(&platform_raw).unwrap();
    assert_eq!(windows.execution.os, "windows");
    assert_eq!(windows.execution.architecture, "x86_64");
    assert_eq!(
        windows.execution.revision,
        "0cf42d39bbe5eaeed477a4d4d9ef241ba54022a1"
    );
    assert_eq!(
        windows.execution.lock_digest,
        e7::sha256(&fs::read(root.join("Cargo.lock")).unwrap())
    );
    assert_eq!(windows.manifest, manifest(&actual).unwrap());
    assert_eq!(
        windows.manifest.evaluation_digest,
        "98d21fe1d543f138b7e414bd5ec915235c2dd5ab3b99be19ba488bce65cf67be"
    );
    assert_eq!(
        windows.manifest.metric_digest,
        "36a411d3e8a5cb2ed842860ed6aeca7ce88280dcd4e787d1687eb5d8faeea2c1"
    );
    assert!(
        e7::compare_platforms(&windows, &windows).is_err(),
        "two Windows executions cannot prove Linux agreement"
    );
}
