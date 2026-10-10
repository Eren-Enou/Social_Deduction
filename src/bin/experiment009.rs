use social_deduction::{experiment007 as e7, experiment009::*};
use std::{error::Error, fs, fs::OpenOptions, io::Write};
fn write<T: serde::Serialize>(v: &T, path: Option<&String>) -> Result<(), Box<dyn Error>> {
    let bytes = serde_json::to_vec(v)?;
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
    let a: Vec<_> = std::env::args().skip(1).collect();
    match a.first().map(String::as_str) {
        Some("scenario") if (5..=6).contains(&a.len())=>{let seed=a[3].parse()?;let policy=serde_json::from_value(serde_json::Value::String(a[4].clone()))?;let c=match a[1].as_str(){"diagnostic"=>diagnostic_config(serde_json::from_value(serde_json::Value::String(a[2].clone()))?,seed,policy),"generated"=>generated_config(&Generator.generate(seed),policy,a[2]=="injected"),_=>return Err("kind must be diagnostic or generated".into())};if a[1]=="generated" && a[2]!="selected" && a[2]!="injected" {return Err("generated variant must be selected or injected".into());}write(&c,a.get(5))},
        Some("run") if (2..=3).contains(&a.len())=>write(&run(serde_json::from_slice(&fs::read(&a[1])?)?)?,a.get(2)),
        Some("controls") if a.len()<=2=>write(&controls()?,a.get(1)),
        Some("counterfactuals") if a.len()<=2=>write(&counterfactuals()?,a.get(1)),
        Some("evaluate") if a.len()<=2=>write(&evaluate()?,a.get(1)),
        Some("replay") if a.len()==2=>write(&e7::PlatformReplay{execution:e7::execution()?,manifest:manifest(&evaluate()?)?},a.get(1)),
        Some("compare") if a.len()==3=>{e7::compare_platforms(&serde_json::from_slice(&fs::read(&a[1])?)?,&serde_json::from_slice(&fs::read(&a[2])?)?)?;println!("actual Windows/Linux canonical replay matches");Ok(())},
        _=>Err("usage: experiment009 scenario diagnostic Family seed Policy [new-config] | scenario generated selected|injected seed Policy [new-config] | run config [new-report] | evaluate [new-evidence] | replay new-manifest | compare windows-manifest linux-manifest".into())
    }
}
