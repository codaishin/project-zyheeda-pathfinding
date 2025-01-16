pub mod channels;
pub mod no_suffix;
pub mod prefix;

use serde::de::Error;
use std::fmt::Display;

pub enum HexColorParseError {
	EmptyString,
	InvalidPrefix(char),
	InvalidSuffix(String),
	Empty(HexColorChanel),
	Incomplete(HexColorChanel),
	FaultyBase(HexColorChanel),
}

impl HexColorParseError {
	pub fn convert_to_serde_error<TError>(self) -> TError
	where
		TError: Error,
	{
		let base = "expected hex coded color (#rrggbb or #rrggbbaa), but";

		match self {
			Self::EmptyString => TError::custom(format!("{base} it was empty")),
			Self::InvalidPrefix(p) => TError::custom(format!("{base} it began with {p}")),
			Self::Empty(c) => TError::custom(format!("{base} {c} was empty")),
			Self::Incomplete(c) => TError::custom(format!("{base} {c} was incomplete")),
			Self::FaultyBase(c) => TError::custom(format!("{base} {c} was not base 16")),
			Self::InvalidSuffix(rest) => {
				TError::custom(format!("{base} it had overflowing characters: \"{rest}\""))
			}
		}
	}
}

pub enum HexColorChanel {
	Red,
	Green,
	Blue,
	Alpha,
}

impl Display for HexColorChanel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Red => f.write_str("Chanel Red (rr)"),
			Self::Green => f.write_str("Chanel Green (gg)"),
			Self::Blue => f.write_str("Chanel Blue (bb)"),
			Self::Alpha => f.write_str("Chanel Alpha (aa)"),
		}
	}
}
