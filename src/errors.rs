use bevy::prelude::*;
use serde_json::error::Error as SerdeJsonError;
use std::{
	error::Error,
	fmt::{Display, Formatter, Result as FmtResult},
	io::Error as IOError,
	str::Utf8Error,
};

#[derive(Debug, TypePath)]
pub enum ReadError {
	IO(IOError),
	ParseChars(Utf8Error),
}

#[derive(Debug, TypePath)]
#[allow(dead_code)]
pub enum LoadError {
	IO(IOError),
	ParseChars(Utf8Error),
	ParseObject(SerdeJsonError),
}

impl Display for LoadError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			LoadError::IO(err) => write!(f, "Failed to read asset file: {}", err),
			LoadError::ParseChars(err) => {
				write!(f, "Invalid character encoding in asset file: {}", err)
			}
			LoadError::ParseObject(err) => write!(f, "Failed to parse asset data: {}", err),
		}
	}
}

impl Error for LoadError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			LoadError::IO(err) => Some(err),
			LoadError::ParseChars(err) => Some(err),
			LoadError::ParseObject(err) => Some(err),
		}
	}
}
