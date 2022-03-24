use std::{error::Error, fs::File};

use markov_chainz::{corpus_cleanup, generate_text, load_model_from_file, train_model};

fn main() -> Result<(), Box<dyn Error>> {
    {
        let mut corpus = std::fs::read_to_string("./corpus/bible_turkish.txt")?;
        corpus_cleanup(&mut corpus);
        let mut f = File::create("./corpus/trained_model.bin")?;
        train_model(&mut f, &corpus)?;
    }

    let mut f = File::open("./corpus/trained_model.bin")?;
    let mut buf = Vec::new();
    let model = load_model_from_file(&mut f, &mut buf)?;

    let t = generate_text(&model, 80, Some("ben"));
    println!("\nGenerated:\n {t}");

    Ok(())
}
