Derives the [`Schema`](zino_core::orm::Schema) trait.

# Attributes on structs

- **`#[schema(reader_name = "name")]`**: The `reader_name` attribute specifies
  the model reader name of database services. The names are defined in the database configuration.
  Default value: **`main`**.

- **`#[schema(writer_name = "name")]`**: The `writer_name` attribute specifies
  the model writer name of database services. The names are defined in the database configuration.
  Default value: **`main`**.

- **`#[schema(table_name = "name")]`**: The `table_name` attribute specifies
  the corresponding table in the database. The default table name is obtained by
  a concatenation of the database namespace and the model name.

- **`#[schema(comment = "doc")]`**: The `comment` attribute specifies
  the documentation of the model. The value will be used in the Avro schema.

# Attributes on struct fields

- **`#[schema(ignore)]`**: The `ignore` annotation is used to skip a particular field
  such that it maps to no database column.

- **`#[schema(type_name = "name")]`**: The `type_name` attribute is used to
  override the Rust data type of the column.

- **`#[schema(column_name = "name")]`**: All column names are assumed to be in **snake-case**.
  You can override it by specifying the `column_name` attribute.

- **`#[schema(column_type = "type")]`**: The column type is derived automatically
  from the mappings of Rust data types for different database drivers.
  You can override it by specifying the `column_type` attribute.

- **`#[schema(length = N)]`**: The `length` attribute specifies
  the fixed string length which will override the `column_type` as `CHAR(N)`.

- **`#[schema(max_length = N)]`**: The `max_length` attribute specifies
  the maximum number of characters which will override the `column_type` as `VARCHAR(N)`.

- **`#[schema(not_null)]`**: The `not_null` annotation is used to indicate that
  the column value can not be `NULL`.

- **`#[schema(default_value = "value")]`**: The `default_value` attribute specifies
  a default column value. If the value is a function, it must be callable as `fn() -> T`.

- **`#[schema(auto_increment)]`**: The `auto_increment` annotation is used to
  automatically fill in default column values.

- **`#[schema(auto_random)]`**: The `auto_increment` annotation is used to
  automatically assign values to a `BIGINT` column.
  Values assigned automatically are **random** and **unique**.
  The feature is only supported by TiDB.

- **`#[schema(index_type = "type")]`**: The `index_type` attribute is used to
  create an index for the database column. Supported values: `btree` | `hash`
  | `gin` | `spatial` | `text` | `unique`.

- **`#[schema(reference = "Model")]`**: The `reference` attribute specifies
  the referenced model to define a relation between two models.
  It will be used for constriaint check and query population.

- **`#[schema(comment = "doc")]`**: The `comment` attribute specifies
  the documentation of the column. The value will be used in the OpenAPI docs.

- **`#[schema(primary_key)]`**: The `primary_key` annotation is used to
  mark a column as the primary key.

- **`#[schema(foreign_key)]`**: The `foreign_key` annotation is used to
  mark a column as the foreign key.

- **`#[schema(read_only)]`**: The `read_only` annotation is used to indicate that
  the column is read-only and can not be modified after creation.

- **`#[schema(write_only)]`**: The `write_only` annotation is used to indicate that
  the column is write-only and can not be seen by frontend users.

- **`#[schema(fuzzy_search)]`**: The `fuzzy_search` annotation is used to indicate that
  the column supports fuzzy search.

- **`#[schema(on_delete = "action")]`**: The `on_delete` attribute specifies
  the referential action for a foreign key when the parent table has a `DELETE` operation.
  Supported values: `cascade` | `restrict`.

- **`#[schema(on_update = "action")]`**: The `on_update` attribute specifies
  the referential action for a foreign key when the parent table has an `UPDATE` operation.
  Supported values: `cascade` | `restrict`.
