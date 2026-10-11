use social_deduction::experiment012::*;
use std::{fs, process::Command};
#[test]
fn headless_cli_writes_exact_config_report_and_refuses_overwrite() {
    let root = std::env::temp_dir().join(format!("social012-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let config = root.join("config.json");
    let report = root.join("report.json");
    let exe = env!("CARGO_BIN_EXE_experiment012");
    let output = Command::new(exe)
        .args(["scenario", "generated", "0", "Targeted"])
        .arg(&config)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let c = generated(0, Treatment::Targeted);
    assert_eq!(fs::read(&config).unwrap(), serde_json::to_vec(&c).unwrap());
    let output = Command::new(exe)
        .arg("run")
        .arg(&config)
        .arg(&report)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let expected = serde_json::to_vec(&run(c).unwrap()).unwrap();
    assert_eq!(fs::read(&report).unwrap(), expected);
    let output = Command::new(exe)
        .arg("run")
        .arg(&config)
        .arg(&report)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(fs::read(&report).unwrap(), expected);
    fs::remove_file(config).unwrap();
    fs::remove_file(report).unwrap();
    fs::remove_dir(root).unwrap();
}
