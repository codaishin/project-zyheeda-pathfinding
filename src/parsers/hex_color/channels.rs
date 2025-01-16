use super::{prefix::Prefix, HexColorChanel, HexColorParseError};
use crate::{parsers::parsed::Parsed, traits::parse::Parse};
use std::str::Chars;

macro_rules! parse_color {
	(@first Alpha, $value:expr) => {{
		let Some(c0) = $value.next() else {
			return Ok((Alpha(u8::MAX), $value));
		};
		c0
	}};
	(@first $chanel:ident, $value:expr) => {{
		let Some(c0) = $value.next() else {
			return Err(HexColorParseError::Empty(HexColorChanel::$chanel));
		};
		c0
	}};
	($chanel:ident, $value:expr) => {{
		let c0 = parse_color!(@first $chanel, $value);
		let Some(c1) = $value.next() else {
			return Err(HexColorParseError::Incomplete(HexColorChanel::$chanel));
		};
		let Ok(c) = u8::from_str_radix(&format!("{c0}{c1}"), 16) else {
			return Err(HexColorParseError::FaultyBase(HexColorChanel::$chanel));
		};

		Ok(($chanel(c), $value))
	}};
}

pub struct Red(pub u8);

impl Parse for Red {
	type TRequired = Parsed<(Prefix,)>;
	type TSource<'a> = Chars<'a>;
	type TError = HexColorParseError;

	fn parse(mut value: Self::TSource<'_>) -> Result<(Self, Self::TSource<'_>), Self::TError> {
		parse_color!(Red, value)
	}
}

pub struct Green(pub u8);

impl Parse for Green {
	type TRequired = Parsed<(Prefix, Red)>;
	type TSource<'a> = Chars<'a>;
	type TError = HexColorParseError;

	fn parse(mut value: Self::TSource<'_>) -> Result<(Self, Self::TSource<'_>), Self::TError> {
		parse_color!(Green, value)
	}
}

pub struct Blue(pub u8);

impl Parse for Blue {
	type TRequired = Parsed<(Prefix, Red, Green)>;
	type TSource<'a> = Chars<'a>;
	type TError = HexColorParseError;

	fn parse(mut value: Self::TSource<'_>) -> Result<(Self, Self::TSource<'_>), Self::TError> {
		parse_color!(Blue, value)
	}
}

pub struct Alpha(pub u8);

impl Parse for Alpha {
	type TRequired = Parsed<(Prefix, Red, Green, Blue)>;
	type TSource<'a> = Chars<'a>;
	type TError = HexColorParseError;

	fn parse(mut value: Self::TSource<'_>) -> Result<(Self, Self::TSource<'_>), Self::TError> {
		parse_color!(Alpha, value)
	}
}
