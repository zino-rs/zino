Derives the [`Entity`](zino_orm::Entity) trait.

# Attributes on struct fields

- **`#[schema(primary_key)]`**: The `primary_key` annotation is used to
  mark a column as the primary key.

- **`#[schema(column_name = "name")]`**: All column names are assumed to be in **snake-case**.
  You can override it by specifying the `column_name` attribute.
