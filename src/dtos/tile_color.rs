use crate::{
	parsers::{
		hex_color::{
			channels::{Alpha, Blue, Green, Red},
			no_suffix::NoSuffix,
			prefix::HexPrefix,
			HexColorParseError,
		},
		string_parser::StringParser,
	},
	traits::load_from::LoadFrom,
};
use bevy::{asset::LoadContext, prelude::*};
use serde::{Deserialize, Deserializer};

#[derive(Debug, PartialEq)]
pub struct TileColor {
	color: Color,
}

impl TileColor {
	fn from_hex_color((_, Red(r), Green(g), Blue(b), Alpha(a), _): HexColor) -> Self {
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
			.parse_color()
			.map(TileColor::from_hex_color)
			.map_err(HexColorParseError::convert_to_serde_error)
	}
}

impl LoadFrom<TileColor> for ColorMaterial {
	fn load_from(TileColor { color }: TileColor, _: &mut LoadContext) -> Self {
		ColorMaterial::from_color(color)
	}
}

#[derive(Debug, PartialEq, Deserialize)]
/// A struct responsible of parsing hex encoded colors `#rrggbb` or `#rrggbbaa`
/// to a bevy color for [`TileColor`]
///
///
/// Let's be honest, the whole parsing process is hugely over-designed
/// and could have been done with way fewer lines of code. We could also
/// have just used bevy's [`Srgba::hex`] function.
///
/// But what is life, if we are not enjoying ourselves?
/// So scroll down and bask in the beauty of my nightly escapades :D
struct _TileColorString {
	color: String,
}

type HexColor = (HexPrefix, Red, Green, Blue, Alpha, NoSuffix);

impl _TileColorString {
	fn parse_color(self) -> Result<HexColor, HexColorParseError> {
		let chars = self.color.chars();

		Ok(StringParser::new(chars)
			.parse::<HexPrefix>()?
			.parse::<Red>()?
			.parse::<Green>()?
			.parse::<Blue>()?
			.parse::<Alpha>()?
			.parse::<NoSuffix>()?
			.unpack())
	}
}
