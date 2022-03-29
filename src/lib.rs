#![allow(clippy::missing_safety_doc)]
#![allow(dead_code)]

// pub mod algo;
pub mod algo_hashing;
pub use algo_hashing::Markor;

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

fn hide_cursor() {
    print!("\x1B[?25l\r");
}

fn unhide_cursor() {
    print!("\x1B[?25h\r\n\n");
}

pub fn corpus_cleanup(corpus: &mut String) {
    *corpus = corpus
        .replace('\n', " ")
        .replace('\t', " ")
        .replace('”', "\"")
        .replace('‟', "\"")
        .replace('\'', "")
        .to_lowercase();
    for p in ['.', '-', ',', '!', '?', '(', '—', ')', '"', '[', ']'] {
        *corpus = corpus.replace(p, &format!(" {} ", p))
    }
}

pub fn decleanup(corpus: &mut String) {
    for p in [
        " .", " -", " ,", " !", " ?", " (", " —", " )", " \"", " [", " ]",
    ] {
        *corpus = corpus.replace(p, &format!("{}", p.chars().nth(1).unwrap()))
    }
}

pub fn get_tokens(corpus: &str) -> Vec<&str> {
    corpus.split_whitespace().collect::<Vec<&str>>()
}
