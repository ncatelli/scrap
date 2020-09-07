use crate::flag::FlagOrValue;
use crate::parsers::ArgumentParser;
use parcel::prelude::v1::*;
use parcel::MatchStatus;

#[test]
fn should_parse_flag_into_flag_or_value() {
    let input = "--no-ask";
    let short_input = "-n";

    assert_eq!(
        Ok(MatchStatus::Match((
            &input[input.len()..],
            FlagOrValue::Flag("no-ask".to_string())
        ))),
        ArgumentParser::new().parse(input)
    );

    assert_eq!(
        Ok(MatchStatus::Match((
            &short_input[short_input.len()..],
            FlagOrValue::Flag("n".to_string())
        ))),
        ArgumentParser::new().parse(short_input)
    );
}
