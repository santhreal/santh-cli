//! Invariant: output formats parse only the documented spellings.

use std::str::FromStr;

use santh_cli::OutputFormat;

#[test]
fn output_format_parses() {
    assert_eq!(OutputFormat::from_str("json"), Ok(OutputFormat::Json));
    assert_eq!(OutputFormat::from_str("sarif"), Ok(OutputFormat::Sarif));
    assert_eq!(OutputFormat::from_str("human"), Ok(OutputFormat::Human));
    assert!(OutputFormat::from_str("xml").is_err());
}
