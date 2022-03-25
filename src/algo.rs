use crate::{hide_cursor, pos_by_word, unhide_cursor, State};

pub fn state_chainer(corpus: &str, _k_word: usize) -> Vec<State> {
    let mut corpus_split = corpus.split_whitespace().collect::<Vec<&str>>();
    let corpus_len = corpus_split.len();

    let mut states = corpus_split
        .iter_mut()
        .map(|w| State {
            word: w,
            next_states: Vec::new(),
        })
        .collect::<Vec<State>>();
    states.sort_by(|s1, s2| s1.word.cmp(s2.word));
    states.dedup_by(|s1, s2| s1.word.eq_ignore_ascii_case(s2.word));

    hide_cursor();
    for (word_i, w) in corpus_split.windows(2).enumerate() {
        let (left, right) = (w[0], w[1]);

        let dst_state = states.binary_search_by(|s| s.word.cmp(right)).unwrap();

        let src_state_i = states.binary_search_by(|s| s.word.cmp(left)).unwrap();
        let src_state = unsafe { states.get_unchecked_mut(src_state_i) };

        match src_state
            .next_states
            .iter_mut()
            .find(|ns| ns.0 == dst_state)
        {
            Some(s) => s.1 += 1,
            None => src_state.next_states.push((dst_state, 1)),
        }
        print!(
            "Processing: {}/{}\r\nCompleted: {}%\x1B[1A\r",
            word_i + 2,
            corpus_len,
            (word_i + 2) * 100 / corpus_len
        );
    }
    unhide_cursor();
    states
}

pub fn state_chainer_sort_insert(corpus: &str, _k_word: usize) -> Vec<State> {
    let corpus_split = corpus.split_whitespace().collect::<Vec<&str>>();
    let corpus_len = corpus_split.len();

    let mut distinct_words = corpus_split.clone();
    distinct_words.sort_unstable();
    distinct_words.dedup();
    let distinct_words_len = distinct_words.len();

    let mut states = distinct_words
        .iter()
        .map(|w| State {
            word: w,
            next_states: Vec::with_capacity(distinct_words_len),
        })
        .collect::<Vec<State>>();

    hide_cursor();
    for (word_i, w) in corpus_split.windows(2).enumerate() {
        let (left, right) = (w[0], w[1]);

        let dst_state = states.binary_search_by(|s| s.word.cmp(right)).unwrap();

        let src_state_i = states.binary_search_by(|s| s.word.cmp(left)).unwrap();
        let src_state = unsafe { states.get_unchecked_mut(src_state_i) };

        match src_state
            .next_states
            .binary_search_by(|s| s.0.cmp(&dst_state))
        {
            Ok(i) => unsafe { src_state.next_states.get_unchecked_mut(i) }.1 += 1,
            Err(i) => src_state.next_states.insert(i, (dst_state, 1)),
        }
        print!(
            "Processing: {}/{}\r\nCompleted: {}%\x1B[1A\r",
            word_i + 2,
            corpus_len,
            (word_i + 2) * 100 / corpus_len
        );
    }
    unhide_cursor();
    states
}

pub fn naive(corpus: &str, _k_word: usize) -> Vec<State> {
    let mut states = Vec::<State>::new();
    let corpus_split = corpus.split_whitespace().collect::<Vec<&str>>();
    let corpus_len = corpus_split.len();

    hide_cursor();
    for (word_i, w) in corpus_split.windows(2).enumerate() {
        let (left, right) = (w[0], w[1]);

        match pos_by_word(&states, left) {
            Some(i) => {
                let mut found = false;

                for (cur_next_state_index, (next_state_index, _)) in
                    unsafe { states.get_unchecked(i) }
                        .next_states
                        .iter()
                        .enumerate()
                {
                    if unsafe { states.get_unchecked(*next_state_index) }.word == right {
                        unsafe {
                            states
                                .get_unchecked_mut(i)
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
            "Processing: {}/{}\r\nCompleted: {}%\x1B[1A\r",
            word_i + 2,
            corpus_len,
            (word_i + 2) * 100 / corpus_len
        );
    }
    unhide_cursor();
    states
}
