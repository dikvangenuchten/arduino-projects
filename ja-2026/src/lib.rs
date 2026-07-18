#![cfg_attr(not(test), no_std)]

pub fn harness_anchor() -> u8 {
    1
}

#[cfg(test)]
mod tests {
    use super::harness_anchor;

    #[test]
    fn harness_intentional_failure() {
        assert_eq!(harness_anchor(), 2);
    }
}
