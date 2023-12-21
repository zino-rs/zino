Derives the [`ModelHooks`](zino_core::model::ModelHooks) trait.

# Attributes on struct fields

- **`#[schema(corelates_with = "field")]`**: The `corelates_with` attribute specifies
  a field from whose referenced model the column value is copied.

- **`#[schema(foreign_field = "field")]`**: The `foreign_field` attribute specifies
  the foreign field name in the corresponding referenced model.
