use markov_chainz::{corpus_cleanup, dump_graph, generate_text, get_tokens, train_model_ngrams};
use std::{error::Error, fs::File};

const CORPUS_DIR: &str = "./d/1646077410.txt";
const MODEL_DIR: &str = "./d/trying.bin";

fn create_dummy_graph() -> Result<(), Box<dyn Error>> {
    let corpus = "i love this and love that and this and that";
    let tokens = get_tokens(corpus);
    let model = train_model_ngrams::<2>(&tokens);
    dump_graph(&model, "graph")?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut corpus = std::fs::read_to_string(CORPUS_DIR)?;
    corpus_cleanup(&mut corpus);
    let tokens = get_tokens(&corpus);
    let model = train_model_ngrams::<2>(&tokens);
    let t = generate_text(&model, 30, None);
    println!("{t}");
    Ok(())
}
