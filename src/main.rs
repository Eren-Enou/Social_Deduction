use serde::Serialize;
use social_deduction::{Baseline, Config, Report, run};
use std::{error::Error, fs::OpenOptions, io::Write};

#[derive(Serialize)]
struct Evaluation {
    runs: usize,
    replay_matches: usize,
    rejected: usize,
    claims: usize,
    verdicts: usize,
    correct: usize,
    reports: Vec<Report>,
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let json = if args.first().map(String::as_str) == Some("evaluate") {
        use Baseline::*;
        let mut result = Evaluation {
            runs: 0,
            replay_matches: 0,
            rejected: 0,
            claims: 0,
            verdicts: 0,
            correct: 0,
            reports: vec![],
        };
        for policies in [
            [Silent; 6],
            [Truthful; 6],
            [Random; 6],
            [Truthful, Truthful, Random, Silent, Random, Silent],
        ] {
            for seed in 0..32 {
                let config = Config { seed, policies };
                let report = run(config.clone());
                let replay = run(config);
                if serde_json::to_vec(&report)? != serde_json::to_vec(&replay)? {
                    return Err("replay mismatch".into());
                }
                result.runs += 1;
                result.replay_matches += 1;
                result.rejected += report
                    .decisions
                    .iter()
                    .filter(|d| d.rejection.is_some())
                    .count();
                result.claims += report.public_claims.len();
                result.verdicts += usize::from(report.verdict.is_some());
                result.correct += usize::from(report.correct == Some(true));
                result.reports.push(report);
            }
        }
        serde_json::to_string_pretty(&result)?
    } else {
        let seed = args.first().map(|s| s.parse()).transpose()?.unwrap_or(0);
        let policy = match args.get(1).map(String::as_str).unwrap_or("truthful") {
            "truthful" => Baseline::Truthful,
            "silent" => Baseline::Silent,
            "random" => Baseline::Random,
            _ => return Err("policy must be truthful, silent, or random".into()),
        };
        serde_json::to_string_pretty(&run(Config {
            seed,
            policies: [policy; 6],
        }))?
    };
    if let Some(path) = args
        .get(2)
        .filter(|_| args[0] != "evaluate")
        .or_else(|| args.get(1).filter(|_| args[0] == "evaluate"))
    {
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        file.write_all(json.as_bytes())?;
    } else {
        println!("{json}");
    }
    Ok(())
}
