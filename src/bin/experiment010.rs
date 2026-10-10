use social_deduction::{experiment007 as e7, experiment010::*};
use std::{error::Error, fs, fs::OpenOptions, io::Write};
fn write<T: serde::Serialize>(v: &T, path: Option<&String>) -> Result<(), Box<dyn Error>> {
    if let Some(p) = path {
        let mut f = OpenOptions::new().write(true).create_new(true).open(p)?;
        serde_json::to_writer(&mut f, v)?;
        f.flush()?;
    } else {
        serde_json::to_writer(std::io::stdout().lock(), v)?;
        println!();
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    match a.first().map(String::as_str){
        Some("run") if (2..=3).contains(&a.len())=>write(&run(serde_json::from_slice(&fs::read(&a[1])?)?)?,a.get(2)),
        Some("scenario") if (5..=6).contains(&a.len())=>{let seed=a[3].parse()?;let policy=serde_json::from_value(serde_json::Value::String(a[4].clone()))?;let c=match a[1].as_str(){"diagnostic"=>diagnostic_config(serde_json::from_value(serde_json::Value::String(a[2].clone()))?,seed,policy,false),"matched"=>diagnostic_config(serde_json::from_value(serde_json::Value::String(a[2].clone()))?,seed,policy,true),"generated"=>generated_config(&Generator.generate(seed),policy,serde_json::from_value(serde_json::Value::String(a[2].clone()))?),_=>return Err("kind must be diagnostic, matched, or generated".into())};write(&c,a.get(5))},
        Some("fixed") if a.len()==6=>{let seed=a[2].parse()?;let policy=serde_json::from_value(serde_json::Value::String(a[3].clone()))?;if a[1]!="feedback"&&a[1]!="acquisition"{return Err("fixed cause must be feedback or acquisition".into());}let configs=fixed_current_configs(seed,policy,a[1]=="feedback");let variant:usize=a[4].parse()?;write(configs.get(variant).ok_or("history variant must be 0 or 1")?,a.get(5))},
        Some("evaluate") if a.len()<=2=>write(&evaluate()?,a.get(1)),
        Some("development") if a.len()<=2=>write(&evaluate_seeds(0..2,0..1)?,a.get(1)),
        Some("replay") if a.len()==2=>write(&e7::PlatformReplay{execution:e7::execution()?,manifest:manifest(&evaluate()?)?},a.get(1)),
        Some("compare") if a.len()==3=>{e7::compare_platforms(&serde_json::from_slice(&fs::read(&a[1])?)?,&serde_json::from_slice(&fs::read(&a[2])?)?)?;println!("actual Windows/Linux canonical replay matches");Ok(())},
        _=>Err("usage: experiment010 scenario diagnostic|matched Family seed Policy [new-config] | scenario generated Clean|MixedFault seed Policy [new-config] | fixed acquisition|feedback seed Policy 0|1 new-config | run config [new-report] | development [new-evidence] | evaluate [new-evidence] | replay new-manifest | compare windows-manifest linux-manifest".into())
    }
}
