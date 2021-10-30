use std::fs;
use std::io;

use serde_json;

mod eval;

fn main() -> io::Result<()> {
    // let mut files = fs::read_dir("/home/sam/repos/rad1/dataset")?
    //     .map(|res| res.map(|e| e.path()))
    //     .collect::<Result<Vec<_>, io::Error>>()?;
    // files.sort();
    // for file in files {
    //     println!("{}", file.display());
    // }
    let mut evaluation = eval::config::EvaluationConfig::default();
    // for i in 0..evaluation.size() {
    //     evaluation[i] = i as i16;
    // }
    let json = serde_json::to_string_pretty(&evaluation)?;
    println!("{}", json);
    Ok(())
}

fn sigmoid(k: f32, score: f32) -> f32 {
    1.0 / (1.0 + (-k * score / 400.0).exp())
}
