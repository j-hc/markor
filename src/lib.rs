use rand::prelude::SliceRandom;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Read, Write};
use std::process::Command;
use std::time::Instant;

type Freq = u32;
type StateIndex = usize;
type TaggedNextState = (StateIndex, Freq);

#[derive(Debug, Serialize, Deserialize)]
pub struct State<'a> {
    word: &'a str,
    next_states: Vec<TaggedNextState>,
}

fn pos_by_word(states: &[State], word: &str) -> Option<StateIndex> {
    states
        .par_iter()
        .position_any(|s| s.word.eq_ignore_ascii_case(word))
}

fn find_by_word<'a>(states: &'a [State], word: &str) -> Option<&'a State<'a>> {
    states
        .par_iter()
        .find_any(|s| s.word.eq_ignore_ascii_case(word))
}

pub fn load_states(corpus: &str, _k_word: usize) -> Vec<State> {
    let mut states = Vec::<State>::new();
    let corpus_split = &corpus.split_whitespace().collect::<Vec<&str>>();
    let corpus_len = corpus_split.len();

    print!("\x1B[?25l\r");
    for (i, w) in corpus_split.windows(2).enumerate() {
        let (left, right) = (w[0], w[1]);

        match pos_by_word(&states, left) {
            Some(i) => {
                let mut found = false;

                for (cur_next_state_index, (next_state_index, _)) in
                    unsafe { states.get_unchecked_mut(i) }
                        .next_states
                        .clone()
                        .iter()
                        .enumerate()
                {
                    if unsafe { states.get_unchecked(*next_state_index) }.word == right {
                        let cur_state = unsafe { states.get_unchecked_mut(i) };
                        unsafe {
                            cur_state
                                .next_states
                                .get_unchecked_mut(cur_next_state_index)
                        }
                        .1 += 1;
                        found = true;
                        break;
                    }
                }
                if !found {
                    let state_i = match pos_by_word(&states, right) {
                        Some(i) => i,
                        None => {
                            let new_state = State {
                                word: right,
                                next_states: Vec::new(),
                            };
                            states.push(new_state);
                            states.len() - 1
                        }
                    };

                    unsafe { states.get_unchecked_mut(i) }
                        .next_states
                        .push((state_i, 1));
                }
            }
            None => {
                let snd_state = State {
                    word: right,
                    next_states: Vec::new(),
                };
                states.push(snd_state);

                let next_states = vec![(states.len() - 1, 1)];
                let fst_state = State {
                    word: left,
                    next_states,
                };
                states.push(fst_state);
            }
        }
        print!(
            "Processing: {i}/{corpus_len}\r\nCompleted: {}%\x1B[1A\r",
            i * 100 / corpus_len
        );
    }
    print!("\x1B[?25h\r\n\n");
    states
}

pub fn generate_text(states: &[State], length: usize, seed: Option<&str>) -> String {
    assert!(!states.is_empty());

    let seed = seed.unwrap_or(states.choose(&mut rand::thread_rng()).unwrap().word);
    let inital_state =
        find_by_word(states, seed).expect("Could not find a state with '{initial_word}'");

    let mut sentence = String::new();

    let mut cur_state = inital_state;
    for _ in 0..length {
        sentence.push(' ');
        sentence.push_str(cur_state.word);

        cur_state = if cur_state.next_states.is_empty() {
            states.choose(&mut rand::thread_rng()).unwrap()
        } else {
            let next_state_i = cur_state
                .next_states
                .choose_weighted(&mut rand::thread_rng(), |s| s.1)
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
        .replace('"', " \" ")
        .replace('"', " \" ")
        .to_lowercase();
    for p in ['.', '-', ',', '!', '?', '(', '—', ')'] {
        *corpus = corpus.replace(p, &format!(" {} ", p))
    }
    // let pre = Regex::new("[^a-zA-ZçÇğĞıİöÖşŞüÜ.?! ]").unwrap();
    // *corpus = pre.replace_all(s, "").to_string();
}

pub fn dump_graph(states: &[State]) -> io::Result<()> {
    let mut f = File::create("graph").unwrap();

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
        .args(["-Tsvg", "graph", "-O"])
        .spawn()?
        .wait()?;
    Ok(())
}

pub fn load_model_from_file<'a>(
    reader: &mut impl Read,
    buf: &'a mut Vec<u8>,
) -> bincode::Result<Vec<State<'a>>> {
    reader.read_to_end(buf).unwrap();
    bincode::deserialize(buf)
}

pub fn train_model(out: &mut impl Write, corpus: &str) -> bincode::Result<()> {
    let i = Instant::now();
    let states = load_states(corpus, 0);
    println!("Training took {:.2} seconds", i.elapsed().as_secs_f32());
    bincode::serialize_into(out, &states)
}
