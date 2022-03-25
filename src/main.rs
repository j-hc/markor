use markov_chainz::algo::state_chainer_sort_insert;
use markov_chainz::{
    corpus_cleanup, dump_graph, dump_model, generate_text, load_model, train_model,
};
use std::io::Read;
use std::{error::Error, fs::File};

const CORPUS_DIR: &str = "./corpus/bible_turkish.txt";
const MODEL_DIR: &str = "./corpus/bible_turkish.bin";

#[allow(dead_code)]
fn create_dummy_graph() -> Result<(), Box<dyn Error>> {
    let corpus = "i love this and love that and this and that";
    let model = train_model(corpus, state_chainer_sort_insert);
    dump_graph(&model, "graph")?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    {
        let mut corpus = std::fs::read_to_string(CORPUS_DIR)?;
        corpus_cleanup(&mut corpus);
        let model = train_model(&corpus, state_chainer_sort_insert);

        let mut f = File::create(MODEL_DIR)?;
        dump_model(&model, &mut f)?;
    }

    let mut f = File::open(MODEL_DIR)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let model = load_model(&buf)?;

    let t = generate_text(&model, 100, Some("ben"));
    println!("\nGenerated:\n{t}");

    Ok(())
}
