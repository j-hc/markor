use markor::{corpus_cleanup, get_tokens, Markor};
use std::{error::Error, fs::File, io::Write};

const CORPUS_DIR: &str = "./corpus/bible_turkish.txt";
const MODEL_DIR: &str = "./corpus/bible_turkish.bin";

const STATE_SIZE: usize = 2;
const SENTENCE_LENGTH: usize = 120;

fn create_dummy_graph() -> std::io::Result<()> {
    let corpus = "i love this and love that and this and that";
    let tokens = get_tokens(corpus);
    let mut markor = Markor::default();
    markor.chain(&tokens, 1);
    markor.dump_graph("graph")?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut corpus = std::fs::read_to_string(CORPUS_DIR)?;
    corpus_cleanup(&mut corpus);
    let tokens = get_tokens(&corpus);

    let mut markor = Markor::default();
    markor.chain(&tokens, STATE_SIZE);

    let mut f = File::create(MODEL_DIR)?;
    let buf = markor.dump_model()?;
    f.write_all(&buf)?;

    {
        let d = std::fs::read(MODEL_DIR)?;
        let markor = Markor::load_model(&d)?;
        let mut generated_text = markor.generate_str(SENTENCE_LENGTH, None);
        markor::decleanup(&mut generated_text);

        println!("{generated_text}");
    }

    Ok(())
}
