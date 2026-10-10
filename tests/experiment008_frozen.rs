use social_deduction::{experiment007 as e7, experiment008::*};
#[test]
fn frozen_primary_windows_manifest_and_secondary_cost_control_replay() {
    let bytes = std::fs::read("evidence/experiment-008.json").unwrap();
    assert_eq!(
        e7::sha256(&bytes),
        "fa852e921f549dbb87344c7d3329a913985608b19a930cd456f0cc1c667a09cc"
    );
    let frozen: Evaluation = serde_json::from_slice(&bytes).unwrap();
    let live = evaluate().unwrap();
    assert_eq!(live, frozen);
    assert_eq!(serde_json::to_vec(&live).unwrap(), bytes);
    let bytes = std::fs::read("evidence/experiment-008-windows.json").unwrap();
    assert_eq!(
        e7::sha256(&bytes),
        "f0aef8d5b460cd2fe5ac68d1fb4429ae64a316c93b39b2c0621edbf9e3d1f381"
    );
    let windows: e7::PlatformReplay = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(windows.execution.os, "windows");
    assert_eq!(
        windows.execution.revision,
        "c2b4ee278cc2d52250b2dcde53ef84c7b6b4bee8"
    );
    assert_eq!(windows.manifest, manifest(&live).unwrap());
    // This checks preserved Windows evidence, not an actual Linux execution.
    let bytes = std::fs::read("evidence/experiment-008-cost-control.json").unwrap();
    assert_eq!(
        e7::sha256(&bytes),
        "9d7447dd3367d9a9c2e3e66cc8e6c7759642db1aa7a244951f34cf17be7d7876"
    );
    let control: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let mut reports = vec![];
    for row in control["reports"].as_array().unwrap() {
        let report: Report = serde_json::from_value(row["report"].clone()).unwrap();
        assert_eq!(
            report.config.action_cost as u64,
            row["declared_action_cost"].as_u64().unwrap()
        );
        assert_eq!(
            e7::sha256(&serde_json::to_vec(&report).unwrap()),
            row["raw_report_sha256"].as_str().unwrap()
        );
        assert_eq!(run(report.config.clone()).unwrap(), report);
        reports.push(report);
    }
    assert_eq!(reports[0].config.incidents, reports[1].config.incidents);
    assert_eq!(reports[0].config.policy, reports[1].config.policy);
    assert_eq!(
        reports[0]
            .ledgers
            .iter()
            .map(|l| u64::from(l.spent))
            .sum::<u64>(),
        10
    );
    assert!(reports[1].ledgers.iter().all(|l| l.spent == 0));
}
