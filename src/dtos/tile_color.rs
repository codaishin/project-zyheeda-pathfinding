use crate::traits::load_from::LoadFrom;
use bevy::{asset::LoadContext, prelude::*};
use serde::{de::Error, Deserialize, Deserializer};
use std::{fmt::Display, str::Chars};

#[derive(Debug, PartialEq)]
pub struct TileColor {
	color: Color,
}

impl TileColor {
	fn from_parsed(_Parsed((_, Red(r), Green(g), Blue(b), Alpha(a), _)): _Parsed<All>) -> Self {
		let color = Color::Srgba(Srgba {
			red: r as f32 / 255.,
			green: g as f32 / 255.,
			blue: b as f32 / 255.,
			alpha: a as f32 / 255.,
		});

		Self { color }
	}
}

impl<'a> Deserialize<'a> for TileColor {
	fn deserialize<TDeserializer>(deserializer: TDeserializer) -> Result<Self, TDeserializer::Error>
	where
		TDeserializer: Deserializer<'a>,
	{
		_TileColorString::deserialize(deserializer)?
			.parse_color::<TDeserializer::Error>()
			.map(TileColor::from_parsed)
	}
}

impl LoadFrom<TileColor> for ColorMaterial {
	fn load_from(TileColor { color }: TileColor, _: &mut LoadContext) -> Self {
		ColorMaterial::from_color(color)
	}
}

#[derive(Debug, PartialEq, Deserialize)]
struct _TileColorString {
	color: String,
}

impl _TileColorString {
	fn parse_color<TError>(self) -> Result<_Parsed<All>, TError>
	where
		TError: Error,
	{
		let chars = &mut self.color.chars();

		_Parsed::parse_prefix(chars)
			.and_then(|parsed| parsed.parse_red(chars))
			.and_then(|parsed| parsed.parse_blue(chars))
			.and_then(|parsed| parsed.parse_green(chars))
			.and_then(|parsed| parsed.parse_alpha(chars))
			.and_then(|parsed| parsed.parse_end_of_string(chars))
			.map_err(_Error::convert_to_error)
	}
}

#[derive(Debug, PartialEq)]
struct Prefix;

#[derive(Debug, PartialEq)]
struct Red(u8);

impl From<u8> for Red {
	fn from(value: u8) -> Self {
		Self(value)
	}
}

impl ChanelLabel for Red {
	fn label() -> _Chanel {
		_Chanel::Red
	}
}

#[derive(Debug, PartialEq)]
struct Green(u8);

impl From<u8> for Green {
	fn from(value: u8) -> Self {
		Self(value)
	}
}

impl ChanelLabel for Green {
	fn label() -> _Chanel {
		_Chanel::Green
	}
}

#[derive(Debug, PartialEq)]
struct Blue(u8);

impl From<u8> for Blue {
	fn from(value: u8) -> Self {
		Self(value)
	}
}

impl ChanelLabel for Blue {
	fn label() -> _Chanel {
		_Chanel::Blue
	}
}

#[derive(Debug, PartialEq)]
struct Alpha(u8);

impl From<u8> for Alpha {
	fn from(value: u8) -> Self {
		Self(value)
	}
}

impl ChanelLabel for Alpha {
	fn label() -> _Chanel {
		_Chanel::Alpha
	}
}

#[derive(Debug, PartialEq)]
struct EndOfString;

type All = (Prefix, Red, Green, Blue, Alpha, EndOfString);

#[derive(Debug, PartialEq)]
struct _Parsed<TEvaluated = ()>(TEvaluated);

impl _Parsed {
	fn parse_prefix(chars: &mut Chars) -> Result<_Parsed<Prefix>, _Error> {
		let Some(prefix) = chars.next() else {
			return Err(_Error::EmptyString);
		};

		if prefix != '#' {
			return Err(_Error::FaultyPrefix(prefix));
		}

		Ok(_Parsed(Prefix))
	}
}

impl _Parsed<Prefix> {
	fn parse_red(self, chars: &mut Chars) -> Result<_Parsed<(Prefix, Red)>, _Error> {
		let _Parsed(prefix) = self;

		Ok(_Parsed((prefix, parse_chanel(chars)?)))
	}
}

impl _Parsed<(Prefix, Red)> {
	fn parse_blue(self, chars: &mut Chars) -> Result<_Parsed<(Prefix, Red, Green)>, _Error> {
		let _Parsed((prefix, red)) = self;

		Ok(_Parsed((prefix, red, parse_chanel(chars)?)))
	}
}

impl _Parsed<(Prefix, Red, Green)> {
	fn parse_green(self, chars: &mut Chars) -> Result<_Parsed<(Prefix, Red, Green, Blue)>, _Error> {
		let _Parsed((prefix, red, green)) = self;

		Ok(_Parsed((prefix, red, green, parse_chanel(chars)?)))
	}
}

impl _Parsed<(Prefix, Red, Green, Blue)> {
	#[allow(clippy::type_complexity)]
	fn parse_alpha(
		self,
		chars: &mut Chars,
	) -> Result<_Parsed<(Prefix, Red, Green, Blue, Alpha)>, _Error> {
		let _Parsed((prefix, red, green, blue)) = self;
		let alpha = parse_chanel::<Alpha>(chars);

		if let Err(_Error::Empty(_Chanel::Alpha)) = alpha {
			return Ok(_Parsed((prefix, red, green, blue, Alpha(255))));
		};

		Ok(_Parsed((prefix, red, green, blue, alpha?)))
	}
}

impl _Parsed<(Prefix, Red, Green, Blue, Alpha)> {
	fn parse_end_of_string(self, chars: &mut Chars) -> Result<_Parsed<All>, _Error> {
		let _Parsed((prefix, red, green, blue, alpha)) = self;

		let rest: String = chars.collect();

		if !rest.is_empty() {
			return Err(_Error::RemainingChars(rest));
		}

		Ok(_Parsed((prefix, red, green, blue, alpha, EndOfString)))
	}
}

#[derive(Clone, Copy)]
enum _Chanel {
	Red,
	Green,
	Blue,
	Alpha,
}

impl Display for _Chanel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			_Chanel::Red => f.write_str("Chanel Red (rr)"),
			_Chanel::Green => f.write_str("Chanel Green (gg)"),
			_Chanel::Blue => f.write_str("Chanel Blue (bb)"),
			_Chanel::Alpha => f.write_str("Chanel Alpha (aa)"),
		}
	}
}

#[derive(Clone)]
enum _Error {
	EmptyString,
	FaultyPrefix(char),
	Empty(_Chanel),
	Incomplete(_Chanel),
	FaultyBase(_Chanel),
	RemainingChars(String),
}

impl _Error {
	fn convert_to_error<TError>(self) -> TError
	where
		TError: Error,
	{
		let base = "expected hex coded color (#rrggbbaa), but";

		match self {
			_Error::EmptyString => TError::custom(format!("{base} it was empty")),
			_Error::FaultyPrefix(p) => TError::custom(format!("{base} it began with {p}")),
			_Error::Empty(c) => TError::custom(format!("{base} {c} was empty")),
			_Error::Incomplete(c) => TError::custom(format!("{base} {c} was incomplete")),
			_Error::FaultyBase(c) => TError::custom(format!("{base} {c} was not base 16")),
			_Error::RemainingChars(rest) => {
				TError::custom(format!("{base} it was followed by \"{rest}\""))
			}
		}
	}
}

fn parse_chanel<TChanel>(chars: &mut Chars) -> Result<TChanel, _Error>
where
	TChanel: From<u8> + ChanelLabel,
{
	let Some(c0) = chars.next() else {
		return Err(_Error::Empty(TChanel::label()));
	};
	let Some(c1) = chars.next() else {
		return Err(_Error::Incomplete(TChanel::label()));
	};
	let Ok(c) = u8::from_str_radix(&format!("{c0}{c1}"), 16) else {
		return Err(_Error::FaultyBase(TChanel::label()));
	};

	Ok(TChanel::from(c))
}

trait ChanelLabel {
	fn label() -> _Chanel;
}
