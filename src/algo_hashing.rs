use rand::prelude::SliceRandom;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::{self, Write};
use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    fmt::Display,
    hash::{BuildHasherDefault, Hash},
};

type Hasher = BuildHasherDefault<XxHash64>;
type Freq = usize;

use twox_hash::XxHash64;

pub trait Chainable: Clone + Eq + Hash {}
impl<T: Eq + Hash + Clone> Chainable for T {}

#[derive(Default, Serialize, Deserialize)]
pub struct Markor<'a, T: 'a + Chainable>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    pub map: HashMap<Cow<'a, [T]>, HashMap<T, Freq, Hasher>, Hasher>,
}

impl<'a, T: Chainable + Serialize + Deserialize<'a>> Markor<'a, T> {
    pub fn dump_model(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(self)
    }

    pub fn load_model(buf: &'a [u8]) -> bincode::Result<Self> {
        bincode::deserialize(buf)
    }
}

impl<'a, T: Chainable + Display> Markor<'a, T> {
    pub fn dump_graph(&self, path: impl AsRef<std::path::Path>) -> io::Result<()> {
        let mut f = std::fs::File::create(&path)?;

        f.write_all(b"digraph Tree {\n")?;
        let mut node_idx = 0;
        for (_, (t, s)) in self.map.iter().enumerate() {
            let mut repr = String::new();
            t.iter().for_each(|e| repr.push_str(&e.to_string()));
            writeln!(f, "    Node_{} [label=\"{}\"]", node_idx, repr)?;

            for (i, (e, freq)) in s.iter().enumerate() {
                writeln!(f, "    Node_{} [label=\"{}\"]", node_idx + i + 1, e)?;
                writeln!(
                    f,
                    "    Node_{} -> Node_{} [label=\"{}\"]",
                    node_idx,
                    node_idx + i + 1,
                    freq
                )?;
            }
            node_idx += s.len() + 1;
        }

        f.write_all(b"\n}")?;
        std::process::Command::new("dot")
            .args(["-Tsvg", path.as_ref().to_str().unwrap(), "-O"])
            .spawn()?
            .wait()?;
        Ok(())
    }
}

impl<'a, T: Chainable> Markor<'a, T> {
    pub fn chain(&mut self, tokens: &'a [T], state_size: usize) {
        tokens
            .windows(state_size + 1)
            .into_iter()
            .for_each(|w| self.add(&w[..state_size], w[state_size].clone()));
    }

    pub fn generate(&self, length: usize, seed: Option<&[T]>) -> Vec<T> {
        let mut chained = Vec::with_capacity(length);
        let mut rng = rand::thread_rng();
        let init = match seed {
            Some(s) => s,
            None => {
                self.map
                    .iter()
                    .collect::<Vec<_>>()
                    .choose(&mut rng)
                    .unwrap()
                    .0
            }
        };

        let mut cur = VecDeque::from(init.to_vec());
        for _ in 0..length {
            let t = self.choose_token(cur.make_contiguous());
            chained.push(t.clone());

            cur.pop_front();
            cur.push_back(t);
        }

        chained
    }

    fn choose_token(&self, state: &[T]) -> T {
        let v = self.map.get(state).unwrap().iter().collect::<Vec<_>>();
        v.choose_weighted(&mut rand::thread_rng(), |t| t.1)
            .unwrap()
            .0
            .clone()
    }

    fn add(&mut self, state: &'a [T], token: T) {
        let f = self
            .map
            .entry(Cow::Borrowed(state))
            .or_insert_with(HashMap::<T, Freq, Hasher>::default)
            .entry(token)
            .or_insert(0);
        *f += 1;
    }
}

impl<'a> Markor<'a, &'a str> {
    pub fn generate_str(&self, length: usize, seed: Option<&[&'a str]>) -> String {
        self.generate(length, seed).join(" ")
    }
}
