use std::borrow::Cow;

use rkyv::{
  Place,
  rancor::Fallible,
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use crate::{utils::OwnedOrRef, with::AsCacheable};

pub struct AsOwned<T = AsCacheable> {
  _inner: T,
}

impl<'a, T, F> ArchiveWith<Cow<'a, F>> for AsOwned<T>
where
  T: ArchiveWith<F>,
  F: ToOwned + 'a,
{
  type Archived = T::Archived;
  type Resolver = T::Resolver;

  fn resolve_with(field: &Cow<F>, resolver: Self::Resolver, out: Place<Self::Archived>) {
    T::resolve_with(field, resolver, out);
  }
}

impl<'a, T, F, S> SerializeWith<Cow<'a, F>, S> for AsOwned<T>
where
  T: SerializeWith<F, S>,
  F: ToOwned + 'a,
  S: Fallible + ?Sized,
{
  fn serialize_with(field: &Cow<F>, s: &mut S) -> Result<Self::Resolver, S::Error> {
    T::serialize_with(field, s)
  }
}

impl<'a, T, F, D> DeserializeWith<T::Archived, Cow<'a, F>, D> for AsOwned<T>
where
  T: ArchiveWith<F> + DeserializeWith<T::Archived, F, D>,
  F: ToOwned<Owned = F> + 'a,
  D: Fallible + ?Sized,
{
  fn deserialize_with(field: &T::Archived, d: &mut D) -> Result<Cow<'a, F>, D::Error> {
    let f = T::deserialize_with(field, d)?;
    Ok(Cow::Owned(f))
  }
}

impl<'a, T, F> ArchiveWith<OwnedOrRef<'a, F>> for AsOwned<T>
where
  T: ArchiveWith<F>,
{
  type Archived = T::Archived;
  type Resolver = T::Resolver;

  fn resolve_with(field: &OwnedOrRef<F>, resolver: Self::Resolver, out: Place<Self::Archived>) {
    T::resolve_with(field.as_ref(), resolver, out);
  }
}

impl<'a, T, F, S> SerializeWith<OwnedOrRef<'a, F>, S> for AsOwned<T>
where
  T: SerializeWith<F, S>,
  S: Fallible + ?Sized,
{
  fn serialize_with(field: &OwnedOrRef<F>, s: &mut S) -> Result<Self::Resolver, S::Error> {
    T::serialize_with(field.as_ref(), s)
  }
}

impl<'a, T, F, D> DeserializeWith<T::Archived, OwnedOrRef<'a, F>, D> for AsOwned<T>
where
  T: ArchiveWith<F> + DeserializeWith<T::Archived, F, D>,
  D: Fallible + ?Sized,
{
  fn deserialize_with(field: &T::Archived, d: &mut D) -> Result<OwnedOrRef<'a, F>, D::Error> {
    let f = T::deserialize_with(field, d)?;
    Ok(OwnedOrRef::Owned(f))
  }
}
