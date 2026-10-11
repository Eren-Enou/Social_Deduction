use super::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
pub fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    fn normalize(v: serde_json::Value) -> Result<serde_json::Value, String> {
        match v {
            serde_json::Value::Number(n) if n.is_f64() => {
                Err("canonical007 rejects floating-point values".into())
            }
            serde_json::Value::Object(m) => {
                let mut sorted = serde_json::Map::new();
                for (k, v) in m.into_iter().collect::<BTreeMap<_, _>>() {
                    sorted.insert(k, normalize(v)?);
                }
                Ok(serde_json::Value::Object(sorted))
            }
            serde_json::Value::Array(values) => Ok(serde_json::Value::Array(
                values
                    .into_iter()
                    .map(normalize)
                    .collect::<Result<_, _>>()?,
            )),
            other => Ok(other),
        }
    }
    serde_json::to_vec(&normalize(
        serde_json::to_value(value).map_err(|e| e.to_string())?,
    )?)
    .map_err(|e| e.to_string())
}
pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn canonical_digest<T: Serialize>(value: &T) -> Result<String, String> {
    let mut digest = Sha256::new();
    digest.update(b"canonical-007-v1\n");
    digest.update(canonical_bytes(value)?);
    Ok(format!("{:x}", digest.finalize()))
}
pub fn report_digests(r: &Report) -> Result<BTreeMap<String, String>, String> {
    let mut d = BTreeMap::new();
    d.insert("full_series".into(), canonical_digest(r)?);
    d.insert(
        "incidents_events".into(),
        canonical_digest(&(
            &r.objective_history,
            r.episodes.iter().map(|e| &e.incident).collect::<Vec<_>>(),
        ))?,
    );
    d.insert("feedback".into(), canonical_digest(&r.feedback)?);
    d.insert(
        "memory_cognition".into(),
        canonical_digest(&(
            r.episodes
                .iter()
                .map(|e| {
                    (
                        &e.memory_before,
                        &e.cognition,
                        &e.ingestions,
                        &e.memory_after,
                    )
                })
                .collect::<Vec<_>>(),
            &r.final_memory,
        ))?,
    );
    d.insert("forecasts".into(), canonical_digest(&r.forecasts)?);
    d.insert(
        "ballots_outcomes".into(),
        canonical_digest(
            &r.episodes
                .iter()
                .map(|e| {
                    (
                        e.incident
                            .decisions
                            .iter()
                            .filter(|d| d.input.phase == crate::Phase::Judgment)
                            .collect::<Vec<_>>(),
                        &e.incident.tally,
                        e.incident.verdict,
                        e.incident.correct,
                    )
                })
                .collect::<Vec<_>>(),
        )?,
    );
    Ok(d)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayManifest {
    pub version: String,
    pub series: usize,
    pub incidents: usize,
    pub evaluation_digest: String,
    pub metric_digest: String,
    pub series_sections: Vec<BTreeMap<String, String>>,
}
impl ReplayManifest {
    pub fn from_evaluation(e: &Evaluation) -> Result<Self, String> {
        Ok(Self {
            version: "replay-manifest-007-v1".into(),
            series: e.rows.len(),
            incidents: e.incidents,
            evaluation_digest: canonical_digest(e)?,
            metric_digest: canonical_digest(&(&e.groups, &e.comparisons))?,
            series_sections: e.rows.iter().map(|r| r.digests.clone()).collect(),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Execution {
    pub os: String,
    pub architecture: String,
    pub rustc: String,
    pub cargo: String,
    pub revision: String,
    pub lock_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformReplay {
    pub execution: Execution,
    pub manifest: ReplayManifest,
}
pub fn execution() -> Result<Execution, String> {
    fn command(cmd: &str, args: &[&str]) -> Result<String, String> {
        let r = std::process::Command::new(cmd)
            .args(args)
            .output()
            .map_err(|e| e.to_string())?;
        if !r.status.success() {
            return Err(format!("{cmd} failed"));
        }
        Ok(String::from_utf8(r.stdout)
            .map_err(|e| e.to_string())?
            .trim()
            .to_string())
    }
    Ok(Execution {
        os: std::env::consts::OS.into(),
        architecture: std::env::consts::ARCH.into(),
        rustc: command("rustc", &["--version"])?,
        cargo: command("cargo", &["--version"])?,
        revision: command("git", &["rev-parse", "HEAD"])?,
        lock_digest: sha256(&std::fs::read("Cargo.lock").map_err(|e| e.to_string())?),
    })
}
pub fn compare_platforms(a: &PlatformReplay, b: &PlatformReplay) -> Result<(), String> {
    let systems = [a.execution.os.as_str(), b.execution.os.as_str()];
    if !(systems.contains(&"windows") && systems.contains(&"linux")) {
        return Err("requires independently executed Windows and Linux manifests".into());
    }
    if a.execution.architecture != b.execution.architecture
        || a.execution.revision != b.execution.revision
        || a.execution.rustc != b.execution.rustc
        || a.execution.cargo != b.execution.cargo
        || a.execution.lock_digest != b.execution.lock_digest
    {
        return Err("replay environments differ in revision/toolchain/architecture/lock".into());
    }
    if a.manifest != b.manifest {
        return Err("canonical replay digests differ".into());
    }
    Ok(())
}
