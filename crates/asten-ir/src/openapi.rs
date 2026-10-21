//! OpenAPI schemas written by hand, for the shapes utoipa's derive cannot
//! express. An internally tagged enum (`#[serde(tag = "kind")]`) with a
//! newtype variant serialises to the inner object's fields plus the tag, which
//! the derive refuses; the helpers here build that shape, and the crates that
//! own such enums (`Atom` here, `Calendar` in asten-time, `StreamEvent` in
//! asten-ls) implement `ToSchema` with them. Each hand-written schema is
//! checked against a real serialised value in asten-api's tests.

use std::borrow::Cow;

use utoipa::openapi::schema::{AllOfBuilder, ObjectBuilder, OneOfBuilder, Schema, Type};
use utoipa::openapi::{Ref, RefOr};
use utoipa::{PartialSchema, ToSchema};

use crate::{Anchor, Atom, Context, Event, Mention, Observation, TemporalConstraint, Value};

/// The value of a fluent, under a name utoipa does not mistake for
/// `serde_json::Value`: the derive reads any field whose type is spelled
/// `Value` as a free-form JSON value, so `Fluent::value` points its schema here.
pub type IrValue = Value;

/// An object that carries the tag, `{ "<tag>": "<value>" }`, with the tag
/// required. A unit variant is exactly this; a struct variant adds its fields.
#[must_use]
pub fn tag_object(tag: &str, value: &str) -> ObjectBuilder {
    ObjectBuilder::new()
        .schema_type(Type::Object)
        .property(
            tag,
            ObjectBuilder::new()
                .schema_type(Type::String)
                .enum_values(Some([value]))
                .build(),
        )
        .required(tag)
}

/// A newtype variant: everything in the inner object, plus the tag.
#[must_use]
pub fn tagged_newtype(tag: &str, value: &str, inner: RefOr<Schema>) -> Schema {
    Schema::AllOf(
        AllOfBuilder::new()
            .item(inner)
            .item(Schema::Object(tag_object(tag, value).build()))
            .build(),
    )
}

/// A `$ref` to the named schema of `T`, as the derive emits for a field.
#[must_use]
pub fn reference<T: ToSchema>() -> RefOr<Schema> {
    RefOr::Ref(Ref::from_schema_name(T::name()))
}

/// Register `T` and the schemas it depends on, as the derive does for a field.
pub fn depend_on<T: ToSchema>(schemas: &mut Vec<(String, RefOr<Schema>)>) {
    schemas.push((T::name().into_owned(), T::schema()));
    T::schemas(schemas);
}

impl PartialSchema for Atom {
    fn schema() -> RefOr<Schema> {
        let tag = "atom";
        RefOr::T(Schema::OneOf(
            OneOfBuilder::new()
                .description(Some(
                    "One perceived unit of evidence, tagged by `atom`. The newtype variants \
                     carry the inner record's fields beside the tag.",
                ))
                .item(tagged_newtype(tag, "mention", reference::<Mention>()))
                .item(tagged_newtype(
                    tag,
                    "observation",
                    reference::<Observation>(),
                ))
                .item(tagged_newtype(tag, "event", reference::<Event>()))
                .item(tagged_newtype(
                    tag,
                    "constraint",
                    reference::<TemporalConstraint>(),
                ))
                .item(Schema::Object(
                    tag_object(tag, "unknown_name")
                        .property("anchor", reference::<Anchor>())
                        .property("surface", String::schema())
                        .property("reading_pos", i64::schema())
                        .required("anchor")
                        .required("surface")
                        .required("reading_pos")
                        .build(),
                ))
                .item(Schema::Object(
                    tag_object(tag, "context_span")
                        .property("anchor", reference::<Anchor>())
                        .property("context", reference::<Context>())
                        .required("anchor")
                        .required("context")
                        .build(),
                ))
                .build(),
        ))
    }
}

impl ToSchema for Atom {
    fn name() -> Cow<'static, str> {
        Cow::Borrowed("Atom")
    }

    fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
        depend_on::<Mention>(schemas);
        depend_on::<Observation>(schemas);
        depend_on::<Event>(schemas);
        depend_on::<TemporalConstraint>(schemas);
        depend_on::<Anchor>(schemas);
        depend_on::<Context>(schemas);
    }
}
