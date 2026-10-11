use social_deduction::{experiment007 as e7, experiment012::*};
use std::{
    error::Error,
    fs,
    fs::OpenOptions,
    io::{BufWriter, Write},
};
fn write<T: serde::Serialize>(value: &T, path: Option<&String>) -> Result<(), Box<dyn Error>> {
    if let Some(path) = path {
        let mut f = BufWriter::with_capacity(
            65536,
            OpenOptions::new().write(true).create_new(true).open(path)?,
        );
        serde_json::to_writer(&mut f, value)?;
        f.flush()?;
    } else {
        serde_json::to_writer(std::io::stdout().lock(), value)?;
        println!();
    }
    Ok(())
}
fn parse<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, serde_json::Error> {
    serde_json::from_value(serde_json::Value::String(s.into()))
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("run") if (2..=3).contains(&args.len())=>write(&run(serde_json::from_slice(&fs::read(&args[1])?)?)?,args.get(2)),
        Some("scenario") if args.len()>=4 && args[1]=="generated" && args.len()<=5=>write(&generated(args[2].parse()?,parse(&args[3])?),args.get(4)),
        Some("scenario") if (5..=6).contains(&args.len()) && args[1]=="diagnostic"=>write(&diagnostic(parse(&args[2])?,args[3].parse()?,parse(&args[4])?),args.get(5)),
        Some("development") if args.len()<=2=>write(&evaluate_seeds(0..2,0..1)?,args.get(1)),
        Some("evaluate") if args.len()<=2=>write(&evaluate()?,args.get(1)),
        Some("replay") if args.len()==2=>write(&e7::PlatformReplay {execution:e7::execution()?,manifest:manifest(&evaluate()?)?},args.get(1)),
        Some("compare") if args.len()==3=>{e7::compare_platforms(&serde_json::from_slice(&fs::read(&args[1])?)?,&serde_json::from_slice(&fs::read(&args[2])?)?)?;println!("actual Windows/Linux canonical replay matches");Ok(())},
        _=>Err("usage: experiment012 scenario generated seed Treatment [new-config] | scenario diagnostic Family seed Listener [new-config] | run config [new-report] | development [new-evidence] | evaluate [new-evidence] | replay new-manifest | compare windows-manifest linux-manifest".into())
    }
}
