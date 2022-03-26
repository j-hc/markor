use crate::{hide_cursor, unhide_cursor, ArbitrarySplit, State};

pub fn state_chainer_ngrams<'a, const N: usize>(tokens: &'a [&'a str]) -> Vec<State<'a, N>>
where
    [&'a str; N * 2]: Sized,
{
    let mut unique_states = tokens.array_windows::<N>().collect::<Vec<_>>();
    unique_states.sort_unstable();
    unique_states.dedup();

    let mut states = unique_states
        .iter()
        .map(|w| State {
            word: **w,
            next_states: Vec::new(),
        })
        .collect::<Vec<_>>();

    let windowed_tokens = tokens.array_windows::<{ N * 2 }>();
    let windowed_tokens_len = windowed_tokens.len();

    hide_cursor();
    for (token_i, w) in windowed_tokens.enumerate() {
        let (left, right) = unsafe { w.n_split::<N>() };

        let dst_state_i = states.binary_search_by(|s| s.word.cmp(right)).unwrap();
        let src_state_i = states.binary_search_by(|s| s.word.cmp(left)).unwrap();

        let src_state = unsafe { states.get_unchecked_mut(src_state_i) };

        match src_state
            .next_states
            .binary_search_by(|s| s.0.cmp(&dst_state_i))
        {
            Ok(i) => unsafe { src_state.next_states.get_unchecked_mut(i) }.1 += 1,
            Err(i) => src_state.next_states.insert(i, (dst_state_i, 1)),
        }
        print!(
            "Processing: {}/{}\r\nCompleted: {}%\x1B[1A\r",
            token_i + N,
            windowed_tokens_len,
            (token_i + N) * 100 / windowed_tokens_len
        );
    }
    unhide_cursor();
    states
}
