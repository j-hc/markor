#![feature(array_windows, generic_const_exprs, let_chains)]
#![allow(clippy::needless_lifetimes)]

pub mod algo;

use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use algo::state_chainer_ngrams;

pub trait ArbitrarySplit<T> {
    unsafe fn mn_split<const M: usize, const N: usize>(&self) -> (&[T; M], &[T; N]);
    unsafe fn n_split<const N: usize>(&self) -> (&[T; N], &[T; N]);
}

impl<T> ArbitrarySplit<T> for [T] {
    unsafe fn mn_split<const M: usize, const N: usize>(&self) -> (&[T; M], &[T; N]) {
        debug_assert_ne!(N, 0);
        debug_assert_ne!(M, 0);
        debug_assert_eq!(self.len(), M + N);
        (
            &*(self.as_ptr() as *const [T; M]),
            &*(self.as_ptr().add(M) as *const [T; N]),
        )
    }

    unsafe fn n_split<const N: usize>(&self) -> (&[T; N], &[T; N]) {
        self.mn_split::<N, N>()
    }
}

type Freq = u32;
type StateIndex = usize;
type TaggedNextState = (StateIndex, Freq);

type Model<'a, const N: usize> = Vec<State<'a, N>>;
type BModel<'a, const N: usize> = &'a [State<'a, N>];

#[derive(Debug, Serialize, Deserialize)]
pub struct State<'a, const N: usize> {
    #[serde(with = "serde_big_array::BigArray", borrow)]
    word: [&'a str; N],
    next_states: Vec<TaggedNextState>,
}

fn hide_cursor() {
    print!("\x1B[?25l\r");
}

fn unhide_cursor() {
    print!("\x1B[?25h\r\n\n");
}

fn find_by_seq<'a, const N: usize>(
    states: &'a [State<'a, N>],
    seq: &[&str],
) -> Option<&'a State<'a, N>> {
    states.iter().find(|s| s.word.eq(seq))
}

pub fn generate_text<'a, const N: usize>(
    model: BModel<'a, N>,
    length: usize,
    seed: Option<&[&str]>,
) -> String {
    assert!(!model.is_empty(), "model is empty");
    assert_eq!(
        model.get(0).unwrap().word.len(),
        seed.map(|s| s.len())
            .unwrap_or_else(|| model.get(0).unwrap().word.len()),
        "seed must be the same length as the ngrams"
    );

    let mut rng = rand::thread_rng();

    let initial_state = if let Some(seed) = seed {
        find_by_seq(model, seed).expect("Could not find a state with the seed")
    } else {
        model.choose(&mut rng).unwrap()
    };

    let mut sentence = String::new();
    let mut cur_state = initial_state;
    for _ in 0..length {
        sentence.push(' ');
        sentence.push_str(&cur_state.word.join(" "));

        cur_state = if cur_state.next_states.is_empty() {
            sentence.push('.');
            model.choose(&mut rng).unwrap()
        } else {
            let next_state_i = cur_state
                .next_states
                .choose_weighted(&mut rng, |s| s.1)
                .unwrap()
                .0;
            unsafe { model.get_unchecked(next_state_i) }
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
}

pub fn dump_graph<const N: usize>(states: BModel<N>, path: impl AsRef<Path>) -> io::Result<()> {
    let mut f = File::create(&path).unwrap();

    f.write_all(b"digraph Tree {\n")?;
    for (i, s) in states.iter().enumerate() {
        writeln!(f, "    Node_{} [label=\"{}\"]", i, s.word.join(" "))?;
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

pub fn get_tokens(corpus: &str) -> Vec<&str> {
    corpus.split_whitespace().collect::<Vec<&str>>()
}

pub fn train_model_ngrams<'a, const N: usize>(tokens: &'a [&'a str]) -> Model<'a, N>
where
    [(); N * 2]: Sized,
{
    let i = Instant::now();
    let states = state_chainer_ngrams::<N>(tokens);
    println!("Trained on {} unique states", states.len());
    println!(
        "Training took {:.2} seconds\nDumping the model..",
        i.elapsed().as_secs_f32()
    );
    states
}

pub fn load_model<const N: usize>(b_model: &[u8]) -> bincode::Result<Model<'_, N>> {
    bincode::deserialize(b_model)
}

pub fn dump_model<const N: usize>(
    model: BModel<'_, N>,
    sink: &mut impl Write,
) -> bincode::Result<()> {
    bincode::serialize_into(sink, model)
}
