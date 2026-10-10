use social_deduction::{experiment007 as e7, experiment009::*};
#[test]
fn frozen_evaluation_and_independent_windows_manifest_replay_exactly() {
    let bytes = std::fs::read("evidence/experiment-009.json").unwrap();
    assert_eq!(
        e7::sha256(&bytes),
        "157019ee5c078b609f34dee295ce4e6415e9411367f13ea77c29aa48183b1872"
    );
    let frozen: Evaluation = serde_json::from_slice(&bytes).unwrap();
    let live = evaluate().unwrap();
    assert_eq!(live, frozen);
    assert_eq!(serde_json::to_vec(&live).unwrap(), bytes);
    let bytes = std::fs::read("evidence/experiment-009-windows.json").unwrap();
    assert_eq!(
        e7::sha256(&bytes),
        "2c38de80a86ba76db54d0b0c918e00f4710730a7e7fc81cc08b3c2b4045d4169"
    );
    let windows: e7::PlatformReplay = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(windows.execution.os, "windows");
    assert_eq!(
        windows.execution.revision,
        "5102065b2e0c8cb855f8af0d53c6d33f620b97f3"
    );
    assert_eq!(
        windows.execution.lock_digest,
        "360931d88271e853ace662b91ff45c6df0537a94c029f7318b7f230812099c92"
    );
    let canonical = manifest(&live).unwrap();
    assert_eq!(windows.manifest, canonical);
    assert_eq!(canonical.series, 1284);
    assert_eq!(canonical.incidents, 3828);
    assert_eq!(canonical.series_sections.len(), canonical.series);
    // Preserved actual Windows execution, not a claim that Linux ran.
}
