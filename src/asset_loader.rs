use crate::{
	errors::{LoadError, ReadError},
	traits::load_from::LoadFrom,
};
use bevy::{
	asset::{io::Reader, AssetLoader, LoadContext},
	prelude::*,
};
use serde::Deserialize;
use std::{marker::PhantomData, str::from_utf8};

pub struct CustomAssetLoader<TAsset, TDto>(PhantomData<(TAsset, TDto)>);

impl<TAsset, TDto> CustomAssetLoader<TAsset, TDto> {
	async fn read<'a>(
		reader: &mut dyn Reader,
		buffer: &'a mut Vec<u8>,
	) -> Result<&'a str, ReadError> {
		reader.read_to_end(buffer).await.map_err(ReadError::IO)?;
		from_utf8(buffer).map_err(ReadError::ParseChars)
	}
}

impl<TAsset, TDto> Default for CustomAssetLoader<TAsset, TDto> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

impl<TAsset, TDto> AssetLoader for CustomAssetLoader<TAsset, TDto>
where
	TAsset: Asset + LoadFrom<TDto>,
	for<'a> TDto: Deserialize<'a> + Sync + Send + 'static,
{
	type Asset = TAsset;
	type Settings = ();
	type Error = LoadError;

	async fn load(
		&self,
		reader: &mut dyn Reader,
		_: &Self::Settings,
		context: &mut LoadContext<'_>,
	) -> Result<Self::Asset, Self::Error> {
		let buffer = &mut vec![];

		let dto = match Self::read(reader, buffer).await {
			Err(ReadError::IO(err)) => return Err(LoadError::IO(err)),
			Err(ReadError::ParseChars(err)) => return Err(LoadError::ParseChars(err)),
			Ok(str) => serde_json::from_str(str),
		};

		match dto {
			Ok(dto) => Ok(TAsset::load_from(dto, context)),
			Err(err) => Err(LoadError::ParseObject(err)),
		}
	}
}
