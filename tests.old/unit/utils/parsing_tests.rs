#[cfg(test)]
mod tests {
    use ig_client::utils::parsing::{ParsedOptionInfo, normalize_text, parse_instrument_name};

    #[test]
    fn test_normalize_text() {
        // Test accent normalization
        assert_eq!(normalize_text("Japón"), "Japan");
        assert_eq!(normalize_text("Alemán"), "Aleman");
        assert_eq!(normalize_text("François"), "Francois");
        assert_eq!(normalize_text("Österreich"), "Osterreich");

        // Test mixed case
        assert_eq!(normalize_text("ESPAÑA"), "ESPANA");
        assert_eq!(normalize_text("Ñandú"), "Nandu");

        // Test no changes needed
        assert_eq!(normalize_text("US Tech 100"), "US Tech 100");
        assert_eq!(normalize_text("Germany 40"), "Germany 40");

        // Test all lowercase accented vowels
        assert_eq!(normalize_text("áàäâã"), "aaaaa");
        assert_eq!(normalize_text("éèëê"), "eeee");
        assert_eq!(normalize_text("íìïî"), "iiii");
        assert_eq!(normalize_text("óòöôõ"), "ooooo");
        assert_eq!(normalize_text("úùüû"), "uuuu");
        assert_eq!(normalize_text("ñç"), "nc");

        // Test all uppercase accented vowels
        assert_eq!(normalize_text("ÁÀÄÂÃ"), "AAAAA");
        assert_eq!(normalize_text("ÉÈËÊ"), "EEEE");
        assert_eq!(normalize_text("ÍÌÏÎ"), "IIII");
        assert_eq!(normalize_text("ÓÒÖÔÕ"), "OOOOO");
        assert_eq!(normalize_text("ÚÙÜÛ"), "UUUU");
        assert_eq!(normalize_text("ÑÇ"), "NC");

        // Test mixed accents in words
        assert_eq!(normalize_text("café"), "cafe");
        assert_eq!(normalize_text("piñata"), "pinata");
        assert_eq!(normalize_text("Zürich"), "Zurich");
    }

    #[test]
    fn test_parsed_option_info_display() {
        // Test with all fields populated
        let info = ParsedOptionInfo {
            asset_name: "US Tech 100".to_string(),
            strike: Some("19200".to_string()),
            option_type: Some("CALL".to_string()),
        };
        assert_eq!(
            format!("{info}"),
            "Asset: US Tech 100, Strike: 19200, Type: CALL"
        );

        // Test with missing strike and option_type
        let info = ParsedOptionInfo {
            asset_name: "Germany 40".to_string(),
            strike: None,
            option_type: None,
        };
        assert_eq!(
            format!("{info}"),
            "Asset: Germany 40, Strike: N/A, Type: N/A"
        );

        // Test with missing option_type only
        let info = ParsedOptionInfo {
            asset_name: "US 500".to_string(),
            strike: Some("4500".to_string()),
            option_type: None,
        };
        assert_eq!(format!("{info}"), "Asset: US 500, Strike: 4500, Type: N/A");
    }

    #[test]
    fn test_parse_instrument_name_wrapper() {
        // These tests call the parse_instrument_name function which already has
        // internal tests, but we're adding them here to ensure coverage in the
        // external test suite as well

        // Standard option format
        let info = parse_instrument_name("US Tech 100 19200 CALL ($1)");
        assert_eq!(info.asset_name, "US Tech 100");
        assert_eq!(info.strike, Some("19200".to_string()));
        assert_eq!(info.option_type, Some("CALL".to_string()));

        // Non-option instrument
        let info = parse_instrument_name("Germany 40");
        assert_eq!(info.asset_name, "Germany 40");
        assert_eq!(info.strike, None);
        assert_eq!(info.option_type, None);

        // Complex format with prefix
        let info = parse_instrument_name("Weekly Germany 40 (Wed)27500 PUT");
        assert_eq!(info.asset_name, "Germany 40");
        assert_eq!(info.strike, Some("27500".to_string()));
        assert_eq!(info.option_type, Some("PUT".to_string()));

        // With accents that should be normalized
        let info = parse_instrument_name("Japón 225 18500 CALL");
        assert_eq!(info.asset_name, "Japan 225");
        assert_eq!(info.strike, Some("18500".to_string()));
        assert_eq!(info.option_type, Some("CALL".to_string()));
    }
}
