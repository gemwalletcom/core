pub struct NumberIncrementer {
    value: u64,
}

impl NumberIncrementer {
    pub fn new(initial_value: u64) -> Self {
        Self { value: initial_value }
    }

    pub fn next_val(&mut self) -> u64 {
        let current = self.value;
        self.value = self.value.wrapping_add(1);
        current
    }

    pub fn current(&self) -> u64 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_incrementer() {
        let mut incrementer = NumberIncrementer::new(10);

        assert_eq!(incrementer.current(), 10);
        assert_eq!(incrementer.next_val(), 10);
        assert_eq!(incrementer.next_val(), 11);
        assert_eq!(incrementer.current(), 12);
    }
}
