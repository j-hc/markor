use markov_chainz::algo::{state_chainer_sort_insert};
use markov_chainz::{corpus_cleanup, generate_text, load_model, train_model};
use std::io::Read;
use std::{error::Error, fs::File};

const CORPUS_DIR: &str = "./corpus/1647210738.txt";
const MODEL_DIR: &str = "./corpus/trained_model_rt.bin";

fn main() -> Result<(), Box<dyn Error>> {
    {
        let mut corpus = std::fs::read_to_string(CORPUS_DIR)?;
        corpus_cleanup(&mut corpus);
        let mut f = File::create(MODEL_DIR)?;
        train_model(&mut f, &corpus, state_chainer_sort_insert)?;
    }

    let mut f = File::open(MODEL_DIR)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let model = load_model(&buf)?;

    let t = generate_text(&model, 100, Some("ben"));
    println!("\nGenerated:\n{t}");

    Ok(())
}
