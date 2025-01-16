pub trait Parse
where
	Self: Sized,
{
	type TRequiresParsed;
	type TSource<'a>;
	type TError;

	fn parse(value: Self::TSource<'_>) -> Result<(Self, Self::TSource<'_>), Self::TError>;
}
