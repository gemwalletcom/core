use std::collections::HashMap;

pub fn country_status() -> HashMap<&'static str, bool> {
    let mut map = HashMap::new();

    let supported = [
        "AD", "AE", "AG", "AI", "AL", "AM", "AO", "AQ", "AR", "AS", "AT", "AU", "AW", "AX", "AZ", "BA", "BB", "BD", "BE", "BF", "BG", "BH", "BI", "BJ", "BL",
        "BM", "BN", "BO", "BQ", "BR", "BS", "BT", "BV", "BW", "BZ", "CA", "CC", "CH", "CI", "CK", "CL", "CM", "CN", "CO", "CR", "CW", "CX", "CY", "CZ", "DE",
        "DJ", "DK", "DM", "DO", "DZ", "EC", "EE", "EG", "ER", "ES", "FI", "FJ", "FK", "FM", "FO", "FR", "GA", "GB", "GD", "GE", "GF", "GG", "GH", "GI", "GL",
        "GM", "GN", "GP", "GQ", "GR", "GS", "GT", "GU", "GW", "GY", "HK", "HM", "HN", "HR", "HT", "HU", "ID", "IE", "IL", "IM", "IN", "IO", "IS", "IT", "JE",
        "JM", "JO", "JP", "KE", "KG", "KH", "KI", "KM", "KN", "KR", "KW", "KY", "KZ", "LA", "LC", "LI", "LK", "LR", "LS", "LT", "LU", "LV", "MA", "MC", "MD",
        "ME", "MF", "MG", "MH", "MK", "MN", "MO", "MP", "MQ", "MR", "MS", "MT", "MU", "MV", "MW", "MX", "MY", "MZ", "NA", "NC", "NE", "NF", "NG", "NL", "NO",
        "NP", "NR", "NU", "NZ", "OM", "PA", "PE", "PF", "PG", "PH", "PK", "PL", "PM", "PN", "PR", "PT", "PW", "PY", "QA", "RE", "RO", "RS", "RW", "SA", "SB",
        "SC", "SE", "SG", "SH", "SI", "SJ", "SK", "SL", "SM", "SN", "SR", "ST", "SV", "SX", "SZ", "TC", "TD", "TF", "TG", "TH", "TJ", "TK", "TL", "TM", "TN",
        "TO", "TR", "TT", "TV", "TW", "TZ", "UA", "UG", "UM", "US", "UY", "UZ", "VA", "VC", "VG", "VI", "VN", "VU", "WF", "WS", "YT", "ZA", "ZM", "ZW",
    ];

    let restricted = [
        "AF", "BY", "CD", "CF", "CU", "EH", "ET", "IQ", "IR", "KP", "LB", "LY", "ML", "MM", "NI", "PS", "RU", "SD", "SO", "SS", "SY", "YE",
    ];

    for country in supported {
        map.insert(country, true);
    }

    for country in restricted {
        map.insert(country, false);
    }

    map
}
