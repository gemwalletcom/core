use rust_decimal::Decimal;
use std::str::FromStr;

// Currency formatting inspired by https://github.com/paupino/rust-decimal

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Currency {
    pub iso_alpha_code: &'static str,
    pub symbol: &'static str,
    pub name: &'static str,
    pub decimal_places: u8,
}
/// ISO 4217 currency definitions
pub mod iso {
    use super::Currency;
    pub const USD: Currency = Currency {
        iso_alpha_code: "USD",
        symbol: "$",
        name: "US Dollar",
        decimal_places: 2,
    };
    pub const EUR: Currency = Currency {
        iso_alpha_code: "EUR",
        symbol: "€",
        name: "Euro",
        decimal_places: 2,
    };
    pub const GBP: Currency = Currency {
        iso_alpha_code: "GBP",
        symbol: "£",
        name: "British Pound",
        decimal_places: 2,
    };
    pub const JPY: Currency = Currency {
        iso_alpha_code: "JPY",
        symbol: "¥",
        name: "Japanese Yen",
        decimal_places: 0,
    };
    pub const CNY: Currency = Currency {
        iso_alpha_code: "CNY",
        symbol: "¥",
        name: "Chinese Yuan",
        decimal_places: 2,
    };
    pub const CAD: Currency = Currency {
        iso_alpha_code: "CAD",
        symbol: "$",
        name: "Canadian Dollar",
        decimal_places: 2,
    };
    pub const AUD: Currency = Currency {
        iso_alpha_code: "AUD",
        symbol: "$",
        name: "Australian Dollar",
        decimal_places: 2,
    };
    pub const CHF: Currency = Currency {
        iso_alpha_code: "CHF",
        symbol: "CHF",
        name: "Swiss Franc",
        decimal_places: 2,
    };
    pub const KRW: Currency = Currency {
        iso_alpha_code: "KRW",
        symbol: "₩",
        name: "South Korean Won",
        decimal_places: 0,
    };
    pub const INR: Currency = Currency {
        iso_alpha_code: "INR",
        symbol: "₹",
        name: "Indian Rupee",
        decimal_places: 2,
    };

    pub const BRL: Currency = Currency {
        iso_alpha_code: "BRL",
        symbol: "R$",
        name: "Brazilian Real",
        decimal_places: 2,
    };
    pub const RUB: Currency = Currency {
        iso_alpha_code: "RUB",
        symbol: "₽",
        name: "Russian Ruble",
        decimal_places: 2,
    };
    pub const MXN: Currency = Currency {
        iso_alpha_code: "MXN",
        symbol: "$",
        name: "Mexican Peso",
        decimal_places: 2,
    };
    pub const ZAR: Currency = Currency {
        iso_alpha_code: "ZAR",
        symbol: "R",
        name: "South African Rand",
        decimal_places: 2,
    };
    pub const SGD: Currency = Currency {
        iso_alpha_code: "SGD",
        symbol: "$",
        name: "Singapore Dollar",
        decimal_places: 2,
    };
    pub const HKD: Currency = Currency {
        iso_alpha_code: "HKD",
        symbol: "$",
        name: "Hong Kong Dollar",
        decimal_places: 2,
    };
    pub const NOK: Currency = Currency {
        iso_alpha_code: "NOK",
        symbol: "kr",
        name: "Norwegian Krone",
        decimal_places: 2,
    };
    pub const SEK: Currency = Currency {
        iso_alpha_code: "SEK",
        symbol: "kr",
        name: "Swedish Krona",
        decimal_places: 2,
    };
    pub const DKK: Currency = Currency {
        iso_alpha_code: "DKK",
        symbol: "kr",
        name: "Danish Krone",
        decimal_places: 2,
    };
    pub const PLN: Currency = Currency {
        iso_alpha_code: "PLN",
        symbol: "zł",
        name: "Polish Zloty",
        decimal_places: 2,
    };

    pub const AED: Currency = Currency {
        iso_alpha_code: "AED",
        symbol: "د.إ",
        name: "UAE Dirham",
        decimal_places: 2,
    };
    pub const SAR: Currency = Currency {
        iso_alpha_code: "SAR",
        symbol: "﷼",
        name: "Saudi Riyal",
        decimal_places: 2,
    };
    pub const EGP: Currency = Currency {
        iso_alpha_code: "EGP",
        symbol: "£",
        name: "Egyptian Pound",
        decimal_places: 2,
    };
    pub const ILS: Currency = Currency {
        iso_alpha_code: "ILS",
        symbol: "₪",
        name: "Israeli Shekel",
        decimal_places: 2,
    };
    pub const TRY: Currency = Currency {
        iso_alpha_code: "TRY",
        symbol: "₺",
        name: "Turkish Lira",
        decimal_places: 2,
    };

    pub const THB: Currency = Currency {
        iso_alpha_code: "THB",
        symbol: "฿",
        name: "Thai Baht",
        decimal_places: 2,
    };
    pub const MYR: Currency = Currency {
        iso_alpha_code: "MYR",
        symbol: "RM",
        name: "Malaysian Ringgit",
        decimal_places: 2,
    };
    pub const IDR: Currency = Currency {
        iso_alpha_code: "IDR",
        symbol: "Rp",
        name: "Indonesian Rupiah",
        decimal_places: 2,
    };
    pub const PHP: Currency = Currency {
        iso_alpha_code: "PHP",
        symbol: "₱",
        name: "Philippine Peso",
        decimal_places: 2,
    };
    pub const VND: Currency = Currency {
        iso_alpha_code: "VND",
        symbol: "₫",
        name: "Vietnamese Dong",
        decimal_places: 0,
    };
    pub const TWD: Currency = Currency {
        iso_alpha_code: "TWD",
        symbol: "$",
        name: "Taiwan Dollar",
        decimal_places: 2,
    };
    pub const NZD: Currency = Currency {
        iso_alpha_code: "NZD",
        symbol: "$",
        name: "New Zealand Dollar",
        decimal_places: 2,
    };

    pub const ARS: Currency = Currency {
        iso_alpha_code: "ARS",
        symbol: "$",
        name: "Argentine Peso",
        decimal_places: 2,
    };
    pub const CLP: Currency = Currency {
        iso_alpha_code: "CLP",
        symbol: "$",
        name: "Chilean Peso",
        decimal_places: 0,
    };
    pub const COP: Currency = Currency {
        iso_alpha_code: "COP",
        symbol: "$",
        name: "Colombian Peso",
        &&cimal_places: 2,
    };
    pub const PEN: Currency = Currency {
        i   symbol: "S/",
        name: "Peruvian Sol",
        decimal_places: 2,
    };

    pub const CZK: Currency = Currency {
        iso_alpha_code: "CZK",
        symbol: "Kč",
        name: "Czech Koruna",
        decimal_places: 2,
    };
    pub const HUF: Currency = Currency {
        iso_alpha_code: "HUF",
        symbol: "Ft",
        name: "Hungarian Forint",
        decimal_places: 2,
    };
    pub const RON: Currency = Currency {
        iso_alpha_code: "RON",
        symbol: "lei",
        name: "Romanian Leu",
        decimal_places: 2,
    };
    pub const BGN: Currency = Currency {
        iso_alpha_code: "BGN",
        symbol: "лв",
        name: "Bulgarian Lev",
        decimal_places: 2,
    };
    pub const HRK: Currency = Currency {
        iso_alpha_code: "HRK",
        symbol: "kn",
        name: "Croatian Kuna",
        decimal_places: 2,
    };

    pub const ISK: Currency = Currency {
        iso_alpha_code: "ISK",
        symbol: "kr",
        name: "Icelandic Krona",
        decimal_places: 0,
    };
    pub const UAH: Currency = Currency {
        iso_alpha_code: "UAH",
        symbol: "₴",
        name: "Ukrainian Hryvnia",
        decimal_places: 2,
    };
    pub const BYN: Currency = Currency {
        iso_alpha_code: "BYN",
        symbol: "Br",
        name: "Belarusian Ruble",
        decimal_places: 2,
    };
    pub const KZT: Currency = Currency {
        iso_alpha_code: "KZT",
        symbol: "₸",
        name: "Kazakhstani Tenge",
        decimal_places: 2,
    };

    pub fn find(code: &str) -> Option<Currency> {
        match code.to_uppercase().as_str() {
            "USD" => Some(USD),
            "EUR" => Some(EUR),
            "GBP" => Some(GBP),
            "JPY" => Some(JPY),
            "CNY" => Some(CNY),
            "CAD" => Some(CAD),
            "AUD" => Some(AUD),
            "CHF" => Some(CHF),
            "KRW" => Some(KRW),
            "INR" => Some(INR),

            "BRL" => Some(BRL),
            "RUB" => Some(RUB),
            "MXN" => Some(MXN),
            "ZAR" => Some(ZAR),
            "SGD" => Some(SGD),
            "HKD" => Some(HKD),
            "NOK" => Some(NOK),
            "SEK" => Some(SEK),
            "DKK" => Some(DKK),
            "PLN" => Some(PLN),

            "AED" => Some(AED),
            "SAR" => Some(SAR),
            "EGP" => Some(EGP),
            "ILS" => Some(ILS),
            "TRY" => Some(TRY),

            "THB" => Some(THB),
            "MYR" => Some(MYR),
            "IDR" => Some(IDR),
            "PHP" => Some(PHP),
            "VND" => Some(VND),
            "TWD" => Some(TWD),
            "NZD" => Some(NZD),

            "ARS" => Some(ARS),
            "CLP" => Some(CLP),
            "COP" => Some(COP),
            "PEN" => Some(PEN),

            "CZK" => Some(CZK),
            "HUF" => Some(HUF),
            "RON" => Some(RON),
            "BGN" => Some(BGN),
            "HRK" => Some(HRK),

            "ISK" => Some(ISK),
            "UAH" => Some(UAH),
            "BYN" => Some(BYN),
            "KZT" => Some(KZT),

            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Money {
    amount: Decimal,
    currency: Currency,
}

impl Money {
    pub fn from_str(amount: &str, currency: Currency) -> Result<Self, rust_decimal::Error> {
        let amount = Decimal::from_str(amount)?;
        Ok(Money { amount, currency })
    }

    pub fn new(amount: Decimal, currency: Currency) -> Self {
        Money { amount, currency }
    }

    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }
}

/// Formatting parameters for money display
#[derive(Debug, Default)]
pub struct Params {
    pub symbol: Option<&'static str>,
    pub code: Option<&'static str>,
    pub rounding: Option<u8>,
    pub thousands_separator: Option<char>,
    pub decimal_separator: Option<char>,
}

/// Formatter for money values
pub struct Formatter;

impl Formatter {
    /// Format a Money value with the given parameters
    pub fn format(money: &Money, params: Params) -> String {
        let amount = money.amount();
        let currency = money.currency();

        let decimal_places = params.rounding.unwrap_or(currency.decimal_places);
        let explicit_rounding = params.rounding.is_some();

        let rounded = amount.round_dp(decimal_places.into());

        let amount_str = Self::format_decimal(rounded, decimal_places, explicit_rounding);

        let formatted_amount = add_thousands_separator(&amount_str, params.thousands_separator.unwrap_or(','), params.decimal_separator.unwrap_or('.'));

        let symbol = params.symbol.unwrap_or(currency.symbol);
        format!("{}{}", symbol, formatted_amount)
    }

    fn format_decimal(decimal: Decimal, max_decimal_places: u8, _explicit_rounding: bool) -> String {
        let formatted = format!("{:.prec$}", decimal, prec = max_decimal_places as usize);

        if max_decimal_places == 2 {
            formatted
        } else if formatted.contains('.') {
            let trimmed = formatted.trim_end_matches('0');
            if trimmed.ends_with('.') {
                trimmed.trim_end_matches('.').to_string()
            } else {
                trimmed.to_string()
            }
        } else {
            formatted
        }
    }
}

fn add_thousands_separator(amount_str: &str, thousands_sep: char, decimal_sep: char) -> String {
    let (integer_part, decimal_part) = amount_str.split_once('.').map_or((amount_str, None), |(int, dec)| (int, Some(dec)));

    let (sign, integer_part) = if let Some(stripped) = integer_part.strip_prefix('-') {
        ("-", stripped)
    } else {
        ("", integer_part)
    };

    let chars: Vec<char> = integer_part.chars().collect();
    let mut formatted_integer = String::new();

    for (i, &ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i).is_multiple_of(3) {
            formatted_integer.push(thousands_sep);
        }
        formatted_integer.push(ch);
    }

    let mut result = format!("{}{}", sign, formatted_integer);
    if let Some(decimal) = decimal_part
        && !decimal.is_empty()
    {
        result.push(decimal_sep);
        result.push_str(decimal);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_currency_find() {
        assert_eq!(iso::find("USD"), Some(iso::USD));
        assert_eq!(iso::find("usd"), Some(iso::USD));
        assert_eq!(iso::find("EUR"), Some(iso::EUR));
        assert_eq!(iso::find("INVALID"), None);
    }

    #[test]
    fn test_money_creation() {
        let money = Money::from_str("100.50", iso::USD).unwrap();
        assert_eq!(money.amount(), Decimal::from_str("100.50").unwrap());
        assert_eq!(money.currency(), &iso::USD);
    }

    #[test]
    fn test_thousands_separator() {
        assert_eq!(add_thousands_separator("1000", ',', '.'), "1,000");
        assert_eq!(add_thousands_separator("1000000", ',', '.'), "1,000,000");
        assert_eq!(add_thousands_separator("1000.50", ',', '.'), "1,000.50");
        assert_eq!(add_thousands_separator("-1000.50", ',', '.'), "-1,000.50");
        assert_eq!(add_thousands_separator("100", ',', '.'), "100");
    }

    #[test]
    fn test_formatter() {
        let money = Money::from_str("1000.50", iso::USD).unwrap();
        let params = Params {
            symbol: Some(iso::USD.symbol),
            rounding: Some(2),
            ..Default::default()
        };

        let formatted = Formatter::format(&money, params);
        assert_eq!(formatted, "$1,000.50");
    }

    #[test]
    fn test_formatter_with_different_currencies() {
        let money_eur = Money::from_str("9999.99", iso::EUR).unwrap();
        let params = Params {
            symbol: Some(iso::EUR.symbol),
            rounding: Some(2),
            ..Default::default()
        };

        let formatted = Formatter::format(&money_eur, params);
        assert_eq!(formatted, "€9,999.99");
    }

    #[test]
    fn test_formatter_custom_precision() {
        let money = Money::from_str("0.123456", iso::USD).unwrap();

        let params_4_digits = Params {
            symbol: Some(iso::USD.symbol),
            rounding: Some(4),
            ..Default::default()
        };

        let formatted = Formatter::format(&money, params_4_digits);
        assert_eq!(formatted, "$0.1235");

        let params_9_digits = Params {
            symbol: Some(iso::USD.symbol),
            rounding: Some(9),
            ..Default::default()
        };

        let formatted = Formatter::format(&money, params_9_digits);
        assert_eq!(formatted, "$0.123456");
    }

    #[test]
    fn test_invalid_money_creation() {
        assert!(Money::from_str("invalid", iso::USD).is_err());
    }

    #[test]
    fn test_zero_value_formatting() {
        let zero_money = Money::from_str("0", iso::USD).unwrap();
        let formatted = Formatter::format(&zero_money, Params::default());
        assert_eq!(formatted, "$0.00");
    }

    #[test]
    fn test_large_number_thousands_separators() {
        let very_large = Money::from_str("999999999999.99", iso::USD).unwrap();
        let formatted = Formatter::format(&very_large, Params::default());
        assert_eq!(formatted, "$999,999,999,999.99");
    }
}
