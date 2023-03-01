use super::ArrowArrayExt;
use crate::{extend::AvroRecordExt, BoxError, Record};
use apache_avro::types::Value;
use datafusion::{arrow::util, dataframe::DataFrame};
use serde::de::DeserializeOwned;

/// Executor trait for [`DataFrame`](datafusion::dataframe::DataFrame).
pub trait DataFrameExecutor {
    /// Executes the `DataFrame` and returns the total number of rows affected.
    async fn execute(self) -> Result<Option<u64>, BoxError>;

    /// Executes the `DataFrame` and parses it as `Vec<Map>`.
    async fn query(self) -> Result<Vec<Record>, BoxError>;

    /// Executes the `DataFrame` and parses it as a `Map`.
    async fn query_one(self) -> Result<Option<Record>, BoxError>;

    /// Executes the `DataFrame` and parses it as `Vec<T>`.
    async fn query_as<T: DeserializeOwned>(self) -> Result<Vec<T>, BoxError>
    where
        Self: Sized,
    {
        let data = self.query().await?;
        let value = data
            .into_iter()
            .map(|record| Value::Map(record.into_avro_map()))
            .collect::<Vec<_>>();
        apache_avro::from_value(&Value::Array(value)).map_err(|err| err.into())
    }

    /// Executes the `DataFrame` and parses it as an instance of type `T`.
    async fn query_one_as<T: DeserializeOwned>(self) -> Result<Option<T>, BoxError>
    where
        Self: Sized,
    {
        if let Some(data) = self.query_one().await? {
            let value = Value::Union(1, Box::new(Value::Map(data.into_avro_map())));
            apache_avro::from_value(&value).map_err(|err| err.into())
        } else {
            Ok(None)
        }
    }

    /// Executes the `DataFrame` and creates a visual representation of record batches.
    async fn output(self) -> Result<String, BoxError>;
}

impl DataFrameExecutor for DataFrame {
    async fn execute(self) -> Result<Option<u64>, BoxError> {
        self.collect().await?;
        Ok(None)
    }

    async fn query(self) -> Result<Vec<Record>, BoxError> {
        let batches = self.collect().await?;
        let mut records = Vec::new();
        let mut max_rows = 0;
        for batch in batches {
            let num_rows = batch.num_rows();
            if num_rows > max_rows {
                records.resize_with(num_rows, Record::new);
                max_rows = num_rows;
            }
            for field in &batch.schema().fields {
                let field_name = field.name().as_str();
                if let Some(array) = batch.column_by_name(field_name) {
                    for i in 0..num_rows {
                        let record = &mut records[i];
                        let value = array.parse_avro_value(i)?;
                        record.push((field_name.to_owned(), value));
                    }
                }
            }
        }
        Ok(records)
    }

    async fn query_one(self) -> Result<Option<Record>, BoxError> {
        let batches = self.limit(0, Some(1))?.collect().await?;
        let mut record = Record::new();
        for batch in batches {
            for field in &batch.schema().fields {
                let field_name = field.name().as_str();
                if let Some(array) = batch.column_by_name(field_name) {
                    let value = array.parse_avro_value(0)?;
                    record.push((field_name.to_owned(), value));
                }
            }
        }
        Ok(Some(record))
    }

    async fn output(self) -> Result<String, BoxError> {
        let batches = self.collect().await?;
        let data = util::pretty::pretty_format_batches(&batches)?;
        Ok(data.to_string())
    }
}
