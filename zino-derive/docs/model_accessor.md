Derives the [`ModelAccessor`](zino_core::orm::ModelAccessor) trait.

# Attributes on structs

- **`#[schema(unique_on = "field_1, field_2, ...")]`**: The `unique_on` attribute specifies
  the composite columns on which the model is considered to be unique.

# Attributes on struct fields

- **`#[schema(aliase = "name")]`**: The `aliase` attribute specifies
  the field name in `ModelAccessor` to be accessed.

- **`#[schema(primary_key)]`**: The `primary_key` annotation is used to
  mark a column as the primary key.

- **`#[schema(snapshot)]`**: The `snapshot` annotation is used to indicate that
  the column should be included in query population. Built-in snapshot fields:
  **`id`** | **`name`** | **`status`** | **`updated_at`** | **`version`**.

- **`#[schema(reference = "Model")]`**: The `reference` attribute specifies
  the referenced model to define a relation between two models.
  It will be used for constriaint check and query population.

- **`#[schema(unique)]`**: The `unique` annotation is used to indicate that
  the column has a unique constraint.

- **`#[schema(not_null)]`**: The `not_null` annotation is used to indicate that
  the column has a not-null constraint. It also prohibits the cases when
  the `String` value is empty or the `Uuid` value is `nil`.

- **`#[schema(nonempty)]`**: The `nonempty` annotation is used to indicate that
  the `String`, `Vec<T>` or `Map` value should be nonempty.

- **`#[schema(validator = "validator")]`**: The `validator` attribute specifies
  a custom validator which is used to validate the string value.
  If the value is a function, it must be callable as `fn() -> T`.

- **`#[schema(format = "format")]`**: The `format` attribute specifies
  the format for a `String` value. Supported values: **`alphabetic`** | **`alphanumeric`**
  | **`ascii`** | **`ascii-alphabetic`** | **`ascii-alphanumeric`** | **`ascii-digit`**
  | **`ascii-hexdigit`** | **`ascii-lowercase`** | **`ascii-uppercase`** | **`credit-card`**
  | **`date`** | **`date-time`** | **`email`** | **`host`** | **`hostname`** | **`ip`**
  | **`ipv4`** | **`ipv6`** | **`lowercase`** | **`numeric`** | **`phone_number`**
  | **`regex`** | **`time`** | **`uppercase`** | **`uri`** | **`uuid`**.

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

- **`#[schema(unique_items)]`**: The `unique_items` annotation is used to indicate that
  the array items should be unique.