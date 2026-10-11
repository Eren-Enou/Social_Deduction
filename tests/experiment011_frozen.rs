use social_deduction::{experiment007 as e7, experiment011::*};
use std::{fs, path::Path};

#[test]
fn frozen_complete_grid_raw_bytes_and_actual_windows_manifest_reproduce() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let raw = fs::read(root.join("evidence/experiment-011.json")).unwrap();
    assert_eq!(raw.len(), 233454880);
    assert_eq!(
        e7::sha256(&raw),
        "001bfe314e3c65e7c43f886a75836e9aceff4153021595fe4ac7992ca5ea262f"
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
        (952, 7616, 952, 64, 96, 2552)
    );
    let actual = evaluate().unwrap();
    assert!(actual == expected, "complete frozen 011 values differ");
    drop(expected);
    assert!(
        serde_json::to_vec(&actual).unwrap() == raw,
        "frozen raw 011 bytes differ"
    );
    drop(raw);
    let platform_raw = fs::read(root.join("evidence/experiment-011-windows.json")).unwrap();
    assert_eq!(platform_raw.len(), 555565);
    assert_eq!(
        e7::sha256(&platform_raw),
        "12e85f955e09c2faad15f846e091e2ace26fc5265f5a2ef3c4025fcadab5e6a9"
    );
    let windows: e7::PlatformReplay = serde_json::from_slice(&platform_raw).unwrap();
    assert_eq!(windows.execution.os, "windows");
    assert_eq!(windows.execution.architecture, "x86_64");
    assert_eq!(
        windows.execution.revision,
        "ba439e8a06a9bb2ad90a94aff0e0fbd9ab54ff61"
    );
    assert_eq!(
        windows.execution.lock_digest,
        e7::sha256(&fs::read(root.join("Cargo.lock")).unwrap())
    );
    assert_eq!(windows.manifest, manifest(&actual).unwrap());
    assert_eq!(
        windows.manifest.evaluation_digest,
        "cb9a03a2104d2e4a2f94beb2ae575d5ba9fd7aeaef7ef86f41bd3aeabc459766"
    );
    assert_eq!(
        windows.manifest.metric_digest,
        "54c2a5587923d2e38f9ae239cfa8c8f719a7c3025e84a28a01c50505085472bb"
    );
    assert!(
        e7::compare_platforms(&windows, &windows).is_err(),
        "two Windows executions cannot prove Linux agreement"
    );
}
