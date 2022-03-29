// use crate::{hide_cursor, unhide_cursor, ArbitrarySplit};
// use rand::prelude::SliceRandom;
// use serde::{Deserialize, Serialize};
// use std::fs::File;
// use std::io::{self, Write};
// use std::path::Path;
// use std::process::Command;
// use std::time::Instant;

// type Freq = u32;
// type StateIndex = usize;
// type TaggedNextState = (StateIndex, Freq);

// type Model<'a, const N: usize> = Vec<State<'a, N>>;
// type BModel<'a, const N: usize> = &'a [State<'a, N>];

// #[derive(Debug, Serialize, Deserialize)]
// pub struct State<'a, const N: usize> {
//     #[serde(with = "serde_big_array::BigArray", borrow)]
//     word: [&'a str; N],
//     next_states: Vec<TaggedNextState>,
// }

// fn find_by_seq<'a, const N: usize>(
//     states: &'a [State<'a, N>],
//     seq: &[&str],
// ) -> Option<&'a State<'a, N>> {
//     states.iter().find(|s| s.word.eq(seq))
// }

// pub fn generate_text<'a, const N: usize>(
//     model: BModel<'a, N>,
//     length: usize,
//     seed: Option<&[&str]>,
// ) -> String {
//     assert!(!model.is_empty(), "model is empty");
//     assert_eq!(
//         model.get(0).unwrap().word.len(),
//         seed.map(|s| s.len())
//             .unwrap_or_else(|| model.get(0).unwrap().word.len()),
//         "seed must be the same length as the ngrams"
//     );

//     let mut rng = rand::thread_rng();

//     let initial_state = if let Some(seed) = seed {
//         find_by_seq(model, seed).expect("Could not find a state with the seed")
//     } else {
//         model.choose(&mut rng).unwrap()
//     };

//     let mut sentence = String::new();
//     let mut cur_state = initial_state;
//     for _ in 0..length {
//         sentence.push(' ');
//         sentence.push_str(&cur_state.word.join(" "));

//         cur_state = if cur_state.next_states.is_empty() {
//             sentence.push('.');
//             model.choose(&mut rng).unwrap()
//         } else {
//             let next_state_i = cur_state
//                 .next_states
//                 .choose_weighted(&mut rng, |s| s.1)
//                 .unwrap()
//                 .0;
//             unsafe { model.get_unchecked(next_state_i) }
//         };
//     }
//     sentence[1..].to_string()
// }

// pub fn dump_graph<const N: usize>(states: BModel<N>, path: impl AsRef<Path>) -> io::Result<()> {
//     let mut f = File::create(&path).unwrap();

//     f.write_all(b"digraph Tree {\n")?;
//     for (i, s) in states.iter().enumerate() {
//         writeln!(f, "    Node_{} [label=\"{}\"]", i, s.word.join(" "))?;
//     }

//     for (i, s) in states.iter().enumerate() {
//         for (_, (child_i, freq)) in s.next_states.iter().enumerate() {
//             writeln!(f, "    Node_{} -> Node_{} [label=\"{}\"]", i, child_i, freq)?;
//         }
//     }
//     f.write_all(b"\n}")?;
//     Command::new("dot")
//         .args(["-Tsvg", path.as_ref().to_str().unwrap(), "-O"])
//         .spawn()?
//         .wait()?;
//     Ok(())
// }

// pub fn load_model<const N: usize>(b_model: &[u8]) -> bincode::Result<Model<'_, N>> {
//     bincode::deserialize(b_model)
// }

// pub fn dump_model<const N: usize>(
//     model: BModel<'_, N>,
//     sink: &mut impl Write,
// ) -> bincode::Result<()> {
//     bincode::serialize_into(sink, model)
// }

// pub fn train_model_ngrams<'a, const N: usize>(tokens: &'a [&'a str]) -> Model<'a, N>
// where
//     [(); N * 2]: Sized,
// {
//     let i = Instant::now();
//     let states = state_chainer_ngrams::<N>(tokens);
//     println!("Trained on {} unique states", states.len());
//     println!(
//         "Training took {:.2} seconds\nDumping the model..",
//         i.elapsed().as_secs_f32()
//     );
//     states
// }
// pub fn state_chainer_ngrams<'a, const N: usize>(tokens: &'a [&'a str]) -> Vec<State<'a, N>>
// where
//     [&'a str; N * 2]: Sized,
// {
//     let mut unique_states = tokens.array_windows::<N>().collect::<Vec<_>>();
//     unique_states.sort_unstable();
//     unique_states.dedup();

//     let mut states = unique_states
//         .iter()
//         .map(|w| State {
//             word: **w,
//             next_states: Vec::new(),
//         })
//         .collect::<Vec<_>>();

//     let windowed_tokens = tokens.array_windows::<{ N * 2 }>();
//     let windowed_tokens_len = windowed_tokens.len();

//     hide_cursor();
//     for (token_i, w) in windowed_tokens.enumerate() {
//         let (left, right) = unsafe { w.n_split::<N>() };

//         let dst_state_i = states.binary_search_by(|s| s.word.cmp(right)).unwrap();
//         let src_state_i = states.binary_search_by(|s| s.word.cmp(left)).unwrap();

//         let src_state = unsafe { states.get_unchecked_mut(src_state_i) };

//         match src_state
//             .next_states
//             .binary_search_by(|s| s.0.cmp(&dst_state_i))
//         {
//             Ok(i) => unsafe { src_state.next_states.get_unchecked_mut(i) }.1 += 1,
//             Err(i) => src_state.next_states.insert(i, (dst_state_i, 1)),
//         }
//         print!(
//             "Processing: {}/{}\r\nCompleted: {}%\x1B[1A\r",
//             token_i + N,
//             windowed_tokens_len,
//             (token_i + N) * 100 / windowed_tokens_len
//         );
//     }
//     unhide_cursor();
//     states
// }
