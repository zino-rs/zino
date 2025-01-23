Derives the [`ModelHooks`](zino_core::model::ModelHooks) trait.

# Attributes on struct fields

- **`#[schema(rename_all = "...")]`**: The `rename_all` attribute is used to
  rename all the fields according to the specific case when decoding the model as a `Map`.
  Supported values: `lowercase` | `UPPERCASE` | `PascalCase` | `camelCase` | `snake_case`
  | `SCREAMING_SNAKE_CASE` | `kebab-case` | `SCREAMING-KEBAB-CASE`.