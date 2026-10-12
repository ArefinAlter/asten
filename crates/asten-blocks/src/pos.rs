//! Fractional indexing for sibling order. Dynodoc stores a dense order key
//! (`pos`) per node and never generates it; this module does. Keys are strings
//! over a base-62 alphabet, compared lexicographically, never ending in the
//! lowest digit so that a key can always be generated between any two.

const ALPHABET: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const BASE: usize = 62;

fn digit(ch: u8) -> usize {
    ALPHABET.iter().position(|c| *c == ch).unwrap_or(0)
}

fn digit_at(key: &str, index: usize) -> Option<usize> {
    key.as_bytes().get(index).map(|byte| digit(*byte))
}

/// A key strictly between `prev` and `next`. `None` on the left is −∞; `None`
/// on the right is +∞. Requires `prev < next` when both are given.
#[must_use]
pub fn between(prev: Option<&str>, next: Option<&str>) -> String {
    let prev = prev.unwrap_or("");
    let mut out = String::new();
    let mut index = 0;
    loop {
        let a = digit_at(prev, index).unwrap_or(0);
        let b = match next {
            None => BASE,
            Some(next) => digit_at(next, index).unwrap_or(BASE),
        };
        if b > a + 1 {
            out.push(char::from(ALPHABET[(a + b) / 2]));
            return out;
        }
        if b == a + 1 {
            // Commit `a` and continue below with the rest of `prev` and +∞ on the right.
            out.push(char::from(ALPHABET[a]));
            index += 1;
            loop {
                let a = digit_at(prev, index).unwrap_or(0);
                if a + 1 < BASE {
                    out.push(char::from(ALPHABET[(a + BASE) / 2]));
                    return out;
                }
                out.push(char::from(ALPHABET[a]));
                index += 1;
            }
        }
        // b == a: shared digit, go deeper.
        out.push(char::from(ALPHABET[a]));
        index += 1;
    }
}

/// `count` keys strictly between `prev` and `next`, in order, chosen by
/// repeated bisection so key length grows with log(count) rather than count.
#[must_use]
pub fn spread(prev: Option<&str>, next: Option<&str>, count: usize) -> Vec<String> {
    let mut out = Vec::with_capacity(count);
    fill(prev, next, count, &mut out);
    out
}

fn fill(prev: Option<&str>, next: Option<&str>, count: usize, out: &mut Vec<String>) {
    if count == 0 {
        return;
    }
    let mid = between(prev, next);
    let left = (count - 1) / 2;
    let right = count - 1 - left;
    fill(prev, Some(&mid), left, out);
    out.push(mid.clone());
    fill(Some(&mid), next, right, out);
}

/// `count` evenly spread keys in order.
#[must_use]
pub fn sequence(count: usize) -> Vec<String> {
    spread(None, None, count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_stay_ordered() {
        let first = between(None, None);
        let after = between(Some(&first), None);
        let before = between(None, Some(&first));
        assert!(before < first && first < after, "{before} {first} {after}");
        let mid = between(Some(&first), Some(&after));
        assert!(first < mid && mid < after, "{first} {mid} {after}");
        let tight = between(Some("a"), Some("b"));
        assert!("a" < tight.as_str() && tight.as_str() < "b", "{tight}");
        let deep = between(Some("az"), Some("b"));
        assert!("az" < deep.as_str() && deep.as_str() < "b", "{deep}");
        let deeper = between(Some("azz"), Some("b"));
        assert!("azz" < deeper.as_str() && deeper.as_str() < "b", "{deeper}");
    }

    #[test]
    fn long_sequences_keep_short_keys() {
        let keys = sequence(10_000);
        assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
        let longest = keys.iter().map(String::len).max().unwrap_or(0);
        assert!(longest <= 24, "longest key is {longest} chars");
        let run = spread(Some("V"), Some("W"), 1_000);
        assert!(run.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(
            run.iter()
                .all(|key| "V" < key.as_str() && key.as_str() < "W")
        );
    }

    #[test]
    fn sequences_are_monotone_and_bisectable() {
        let keys = sequence(200);
        for pair in keys.windows(2) {
            assert!(pair[0] < pair[1]);
            let mid = between(Some(&pair[0]), Some(&pair[1]));
            assert!(
                pair[0] < mid && mid < pair[1],
                "{} {} {}",
                pair[0],
                mid,
                pair[1]
            );
        }
    }
}
