use clap::builder::{StringValueParser, TypedValueParser};
use primitive_types::U256;

#[derive(Debug, Clone, Copy)]
pub struct U256Parser;

impl TypedValueParser for U256Parser {
    type Value = U256;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let inner = StringValueParser::new();
        let val = inner.parse_ref(cmd, arg, value)?;

        U256::from_dec_str(&val).map_err(|err| {
            clap::Error::raw(clap::error::ErrorKind::InvalidValue, format!("{err:?}"))
        })
    }
}
