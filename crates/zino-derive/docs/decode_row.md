Derives the [`DecodeRow`](zino_orm::DecodeRow) trait.

# Attributes on structs

- **`#[schema(auto_coalesce)]`**: The `auto_coalesce` annotation is used to
  coalesce the default values of model fields when decoding a row in the database.

# Attributes on struct fields

- **`#[schema(ignore)]`**: The `ignore` annotation is used to skip a particular field
  such that it does not need to be decoded.

- **`#[schema(write_only)]`**: The `write_only` annotation is used to indicate that
  the column is write-only and therefore does not need to be decoded.
