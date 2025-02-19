Derives the [`ModelAccessor`](zino_orm::ModelAccessor) trait.

# Attributes on structs

- **`#[schema(auto_rename)]`**: The `auto_rename` annotation is used to
  rename the field name automatically when fetching data of the referenced model.

- **`#[schema(unique_on = "field_1, field_2, ...")]`**: The `unique_on` attribute specifies
  the composite columns on which the model is considered to be unique.

# Attributes on struct fields

- **`#[schema(aliase = "name")]`**: The `aliase` attribute specifies
  the field name in `ModelAccessor` to be accessed.

- **`#[schema(primary_key)]`**: The `primary_key` annotation is used to
  mark a column as the primary key.

- **`#[schema(protected)]`**: The `protected` annotation is used to indicate that
  the column should not be included in a query population.

- **`#[schema(snapshot)]`**: The `snapshot` annotation is used to indicate that
  the column should be included in a query population. Built-in snapshot fields:
  `id` | `name` | `status` | `updated_at` | `version`.

- **`#[schema(reference = "Model")]`**: The `reference` attribute specifies
  the referenced model to define a relation between two models.
  It will be used for constraint check and query population.

- **`#[schema(fetch_as = "field")]`**: The `fetch_as` attribute specifies
  the field name when fetching data of the referenced model.

- **`#[schema(translate_as = "field")]`**: The `translate_as` attribute specifies
  the field name when translating the model data.

- **`#[schema(unique)]`**: The `unique` annotation is used to indicate that
  the column value should be unique in the table.

- **`#[schema(not_null)]`**: The `not_null` annotation is used to indicate that
  the column has a not-null constraint. It also prohibits the case when
  the `Uuid` value is `nil`.

- **`#[schema(nonempty)]`**: The `nonempty` annotation is used to indicate that
  the `String`, `Vec<T>` or `Map` value should be nonempty.

- **`#[schema(validator = "validator")]`**: The `validator` attribute specifies
  a custom validator which is used to validate the string value.
  If the value is a function, it must be callable as `fn() -> T`.

- **`#[schema(format = "format")]`**: The `format` attribute specifies
  the format of a `String` value. Supported values: `alphabetic` | `alphanumeric`
  | `ascii` | `ascii-alphabetic` | `ascii-alphanumeric` | `ascii-digit`
  | `ascii-hexdigit` | `ascii-lowercase` | `ascii-uppercase` | `credit-card`
  | `date` | `date-time` | `email` | `host` | `hostname` | `ip` | `ipv4` | `ipv6`
  | `lowercase` | `numeric` | `phone-number` | `regex` | `time` | `uppercase`
  | `uri` | `uuid`.

- **`#[schema(locale = "lang")]`**: The `locale` attribute specifies
  the language for the column value. It will be used in data mocking.
  Supported values: `en` | `es` | `de` | `fr` | `zh`.

- **`#[schema(enum_values = "value1 | value2 | ...")]`**: The `enum_values` attribute specifies
  the enumerated values for a `String` or `Vec<String>` value.

- **`#[schema(length = N)]`**: The `length` attribute specifies
  the fixed length for a `String` value.

- **`#[schema(max_length = N)]`**: The `max_length` attribute specifies
  the maximum number of characters for a `String` value.

- **`#[schema(min_length = N)]`**: The `max_length` attribute specifies
  the minimum number of characters for a `String` value.

- **`#[schema(max_items = N)]`**: The `max_items` attribute specifies
  the maximum number of items for a `Vec<T>` value.

- **`#[schema(min_items = N)]`**: The `max_items` attribute specifies
  the minimum number of items for a `Vec<T>` value.

- **`#[schema(scale = N)]`**: The `scale` attribute specifies
  the scale for a `Decimal` value.

- **`#[schema(unique_items)]`**: The `unique_items` annotation is used to indicate that
  the array items should be unique.

- **`#[schema(minimum = integer)]`**: The `minimum` attribute specifies
  the minimum integer for the column value.

- **`#[schema(maximum = integer)]`**: The `maximum` attribute specifies
  the maximum integer for the column value.

- **`#[schema(less_than = "value")]`**: The `less_than` attribute specifies
  a comparison relation in which the column value is less than another column.
  If the value is a function, it must be callable as `fn() -> T`.

- **`#[schema(greater_than = "value")]`**: The `less_than` attribute specifies
  a comparison relation in which the column value is greater than another column.
  If the value is a function, it must be callable as `fn() -> T`.
