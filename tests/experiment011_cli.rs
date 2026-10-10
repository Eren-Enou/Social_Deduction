use std::{fs, process::Command};
#[test]
fn exact_config_output_and_create_once_cli() {
    let root =
        std::env::temp_dir().join(format!("social-deduction-011-cli-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let output = root.join("config.json");
    let args = [
        "scenario",
        "diagnostic",
        "AccurateFirsthand",
        "0",
        "ProvenanceAwareSocial",
    ];
    let generated = Command::new(env!("CARGO_BIN_EXE_experiment011"))
        .args(args)
        .arg(&output)
        .output()
        .unwrap();
    assert!(generated.status.success());
    let raw = fs::read(&output).unwrap();
    let expected = social_deduction::experiment011::diagnostic(
        social_deduction::experiment011::Diagnostic::AccurateFirsthand,
        0,
        social_deduction::experiment011::SocialKind::ProvenanceAwareSocial,
    );
    assert_eq!(raw, serde_json::to_vec(&expected).unwrap());
    assert!(
        !Command::new(env!("CARGO_BIN_EXE_experiment011"))
            .args(args)
            .arg(&output)
            .output()
            .unwrap()
            .status
            .success()
    );
    assert_eq!(fs::read(&output).unwrap(), raw);
    fs::remove_file(&output).unwrap();
    fs::remove_dir(&root).unwrap();
}
