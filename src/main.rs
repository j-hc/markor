#![feature(slice_partition_dedup)]

use markov_chainz::{
    corpus_cleanup, dump_graph, dump_model, generate_text, get_tokens, load_model,
    train_model_ngrams,
};
use std::{error::Error, fs::File};

const CORPUS_DIR: &str = "./corpus/bible_turkish.txt";
const MODEL_DIR: &str = "./d/trying.bin";

const STATE_SIZE: usize = 2;

fn create_dummy_graph() -> Result<(), Box<dyn Error>> {
    let corpus = "i love this and love that and this and that";
    let tokens = get_tokens(corpus);
    let model = train_model_ngrams::<STATE_SIZE>(&tokens);
    dump_graph(&model, "graph")?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut corpus = std::fs::read_to_string(CORPUS_DIR)?;
    corpus_cleanup(&mut corpus);
    let tokens = get_tokens(&corpus);
    let model = train_model_ngrams::<STATE_SIZE>(&tokens);

    let mut model_file = File::create(MODEL_DIR)?;
    dump_model(&model, &mut model_file)?;
    let t = generate_text(&model, 30, None);
    println!("{t}");

    // let model = std::fs::read(MODEL_DIR)?;
    // let model = load_model::<STATE_SIZE>(&model)?;
    // let t = generate_text(&model, 30, None);
    // println!("{t}");
    Ok(())
}
