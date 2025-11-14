use primitives::chain::Chain;

const BITCOINCASH_PREFIX: &str = "bitcoincash:";

pub struct Address {
    value: String,
    chain: Chain,
}

impl Address {
    pub fn new(value: impl Into<String>, chain: Chain) -> Self {
        Self { value: value.into(), chain }
    }

    pub fn short(&self) -> &str {
        match self.chain {
            Chain::BitcoinCash => self.value.strip_prefix(BITCOINCASH_PREFIX).unwrap_or(&self.value),
            _ => &self.value,
        }
    }

    pub fn full(&self) -> String {
        match self.chain {
            Chain::BitcoinCash if !self.value.starts_with(BITCOINCASH_PREFIX) => {
                format!("{}{}", BITCOINCASH_PREFIX, self.value)
            }
            _ => self.value.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short() {
        let addr = Address::new("bitcoincash:qqm3kh5j8ptj2y4ryglk0j83t6jkcjk7x52kgzvh4q", Chain::BitcoinCash);
        assert_eq!(addr.short(), "qqm3kh5j8ptj2y4ryglk0j83t6jkcjk7x52kgzvh4q");

        let addr = Address::new("qqm3kh5j8ptj2y4ryglk0j83t6jkcjk7x52kgzvh4q", Chain::BitcoinCash);
        assert_eq!(addr.short(), "qqm3kh5j8ptj2y4ryglk0j83t6jkcjk7x52kgzvh4q");

        let addr = Address::new("bc1qinput", Chain::Bitcoin);
        assert_eq!(addr.short(), "bc1qinput");
    }

    #[test]
    fn test_full() {
        let addr = Address::new("qqm3kh5j8ptj2y4ryglk0j83t6jkcjk7x52kgzvh4q", Chain::BitcoinCash);
        assert_eq!(addr.full(), "bitcoincash:qqm3kh5j8ptj2y4ryglk0j83t6jkcjk7x52kgzvh4q");

        let addr = Address::new("bitcoincash:qqm3kh5j8ptj2y4ryglk0j83t6jkcjk7x52kgzvh4q", Chain::BitcoinCash);
        assert_eq!(addr.full(), "bitcoincash:qqm3kh5j8ptj2y4ryglk0j83t6jkcjk7x52kgzvh4q");

        let addr = Address::new("bc1qinput", Chain::Bitcoin);
        assert_eq!(addr.full(), "bc1qinput");
    }
}
