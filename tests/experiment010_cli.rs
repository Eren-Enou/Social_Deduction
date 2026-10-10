use social_deduction::experiment010::*;
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn buffered_cli_preserves_exact_json_and_refuses_to_overwrite() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = root.join("target").join(format!(
        "experiment010-cli-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&dir).unwrap();
    let config = dir.join("config.json");
    let output = dir.join("report.json");
    let mut c = diagnostic_config(
        Diagnostic::AccurateRecurring,
        0,
        PolicyKind::ExperienceAdjusted,
        false,
    );
    c.incidents.truncate(1);
    c.feedback.retain(|f| f.after_episode == 0);
    let expected = serde_json::to_vec(&run(c.clone()).unwrap()).unwrap();
    fs::write(&config, serde_json::to_vec(&c).unwrap()).unwrap();
    let invoke = || {
        Command::new(env!("CARGO_BIN_EXE_experiment010"))
            .current_dir(root)
            .arg("run")
            .arg(&config)
            .arg(&output)
            .output()
            .unwrap()
    };
    let first = invoke();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(fs::read(&output).unwrap(), expected);
    let second = invoke();
    assert!(!second.status.success());
    assert_eq!(fs::read(&output).unwrap(), expected);
    fs::remove_file(config).unwrap();
    fs::remove_file(output).unwrap();
    fs::remove_dir(dir).unwrap();
}
