use social_deduction::experiment006::{
    Config, Fixture, ListenerKind, evaluate, fixture_config, run,
};
use std::{error::Error, fs, fs::OpenOptions, io::Write};
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (json, path) = match args.first().map(String::as_str) {
        Some("example") if args.len() <= 2 => (
            serde_json::to_string_pretty(&fixture_config(Fixture::Accurate, 0, ListenerKind::Credibility))?,
            args.get(1),
        ),
        Some("evaluate") if args.len() <= 2 => {
            (serde_json::to_string_pretty(&evaluate()?)?, args.get(1))
        }
        Some("run") if (2..=3).contains(&args.len()) => {
            let c: Config = serde_json::from_str(&fs::read_to_string(&args[1])?)?;
            (serde_json::to_string_pretty(&run(c)?)?, args.get(2))
        }
        _ => return Err(
            "usage: experiment006 example [new-config.json] | evaluate [new-report.json] | run config.json [new-report.json]"
                .into(),
        ),
    };
    if let Some(path) = path {
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        file.write_all(json.as_bytes())?;
    } else {
        println!("{json}");
    }
    Ok(())
}
