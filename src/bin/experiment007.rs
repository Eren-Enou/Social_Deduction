use social_deduction::{experiment006 as e6, experiment007::*};
use std::{error::Error, fs, fs::OpenOptions, io::Write};
fn write<T: serde::Serialize>(value: &T, path: Option<&String>) -> Result<(), Box<dyn Error>> {
    let bytes = serde_json::to_vec(value)?;
    if let Some(path) = path {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?
            .write_all(&bytes)?;
    } else {
        println!("{}", String::from_utf8(bytes)?);
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("example") if args.len()<=2=>write(&diagnostic_config(Diagnostic::RewardFalse,0,VerificationKind::ConflictAware,e6::ListenerKind::Credibility),args.get(1)),
        Some("scenario") if (6..=7).contains(&args.len()) => {
            let seed = args[3].parse()?;
            let acceptance = serde_json::from_value(serde_json::Value::String(args[4].clone()))?;
            let listener = serde_json::from_value(serde_json::Value::String(args[5].clone()))?;
            let family = serde_json::Value::String(args[2].clone());
            let config = match args[1].as_str() {
                "generated" => generated_config(&Generator.generate(seed), serde_json::from_value(family)?, acceptance, listener),
                "diagnostic" => diagnostic_config(serde_json::from_value(family)?, seed, acceptance, listener),
                _ => return Err("scenario kind must be generated or diagnostic".into()),
            };
            write(&config, args.get(6))
        },
        Some("run") if (2..=3).contains(&args.len())=>write(&run(serde_json::from_slice(&fs::read(&args[1])?)?)?,args.get(2)),
        Some("evaluate") if args.len()<=2=>write(&evaluate()?,args.get(1)),
        Some("replay") if args.len()==2=>write(&PlatformReplay{execution:execution()?,manifest:ReplayManifest::from_evaluation(&evaluate()?)?},args.get(1)),
        Some("compare") if args.len()==3=>{
            let a=serde_json::from_slice(&fs::read(&args[1])?)?;let b=serde_json::from_slice(&fs::read(&args[2])?)?;
            compare_platforms(&a,&b)?;println!("Windows/Linux canonical replay matches");Ok(())
        },
        _=>Err("usage: experiment007 example [new-config] | scenario generated|diagnostic Family seed Frozen006|ConflictAware Frozen005|Credibility [new-config] | run config [new-report] | evaluate [new-evidence] | replay new-manifest | compare windows-manifest linux-manifest".into()),
    }
}
