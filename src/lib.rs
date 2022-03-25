use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::time::Instant;

type Freq = u32;
type StateIndex = usize;
type TaggedNextState = (StateIndex, Freq);

pub mod algo;

#[derive(Debug, Serialize, Deserialize)]
pub struct State<'a> {
    word: &'a str,
    next_states: Vec<TaggedNextState>,
}

fn hide_cursor() {
    print!("\x1B[?25l\r");
}

fn unhide_cursor() {
    print!("\x1B[?25h\r\n\n");
}

fn pos_by_word(states: &[State], word: &str) -> Option<StateIndex> {
    states
        .iter()
        .position(|s| s.word.eq_ignore_ascii_case(word))
}

fn find_by_word<'a>(states: &'a [State], word: &str) -> Option<&'a State<'a>> {
    states.iter().find(|s| s.word.eq_ignore_ascii_case(word))
}

pub fn generate_text(states: &[State], length: usize, seed: Option<&str>) -> String {
    assert!(!states.is_empty());
    let mut rng = rand::thread_rng();

    let seed = seed.unwrap_or(states.choose(&mut rng).unwrap().word);
    let inital_state = find_by_word(states, seed).expect("Could not find a state with the seed");

    let mut sentence = String::new();
    let mut cur_state = inital_state;
    for _ in 0..length {
        sentence.push(' ');
        sentence.push_str(cur_state.word);

        cur_state = if cur_state.next_states.is_empty() {
            sentence.push('.');
            states.choose(&mut rng).unwrap()
        } else {
            let next_state_i = cur_state
                .next_states
                .choose_weighted(&mut rng, |s| s.1)
                .unwrap()
                .0;
            unsafe { states.get_unchecked(next_state_i) }
        };
    }
    sentence[1..].to_string()
}

pub fn corpus_cleanup(corpus: &mut String) {
    *corpus = corpus
        .replace('\n', " ")
        .replace('\t', " ")
        .replace('”', "\"")
        .replace('‟', "\"")
        .replace('\'', "")
        .to_lowercase();
    for p in ['.', '-', ',', '!', '?', '(', '—', ')', '"'] {
        *corpus = corpus.replace(p, &format!(" {} ", p))
    }
    // let pre = Regex::new("[^a-zA-ZçÇğĞıİöÖşŞüÜ.?! ]").unwrap();
    // *corpus = pre.replace_all(s, "").to_string();
}

pub fn dump_graph(states: &[State], path: impl AsRef<Path>) -> io::Result<()> {
    let mut f = File::create(&path).unwrap();

    f.write_all(b"digraph Tree {\n")?;
    for (i, s) in states.iter().enumerate() {
        writeln!(f, "    Node_{} [label=\"{}\"]", i, s.word)?;
    }

    for (i, s) in states.iter().enumerate() {
        for (_, (child_i, freq)) in s.next_states.iter().enumerate() {
            writeln!(f, "    Node_{} -> Node_{} [label=\"{}\"]", i, child_i, freq)?;
        }
    }
    f.write_all(b"\n}")?;
    Command::new("dot")
        .args(["-Tsvg", path.as_ref().to_str().unwrap(), "-O"])
        .spawn()?
        .wait()?;
    Ok(())
}

pub fn load_model(buf: &[u8]) -> bincode::Result<Vec<State>> {
    bincode::deserialize(buf)
}

pub fn train_model<T>(corpus: &str, trainer: T) -> Vec<State>
where
    T: Fn(&str, usize) -> Vec<State>,
{
    let i = Instant::now();
    let states = trainer(corpus, 0);
    println!("Trained on {} unique states", states.len());
    println!(
        "Training took {:.2} seconds\nDumping the model..",
        i.elapsed().as_secs_f32()
    );
    states
}

pub fn dump_model(model: &[State], sink: &mut impl Write) -> bincode::Result<()> {
    bincode::serialize_into(sink, model)
}
