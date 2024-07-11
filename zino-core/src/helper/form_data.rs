use crate::{error::Error, extension::JsonObjectExt, file::NamedFile, Map};
use multer::Multipart;
use serde::de::DeserializeOwned;

/// Parses the `multipart/form-data` as an instance of type `T` and a list of files.
pub(crate) async fn parse_form_data<T: DeserializeOwned>(
    mut multipart: Multipart<'_>,
) -> Result<(T, Vec<NamedFile>), Error> {
    let mut data = Map::new();
    let mut files = Vec::new();
    while let Some(field) = multipart.next_field().await? {
        if field.file_name().is_some() {
            let file = NamedFile::try_from_multipart_field(field).await?;
            files.push(file);
        } else if let Some(name) = field.name() {
            let key = name.to_owned();
            let value = field.text().await?;
            data.upsert(key, value);
        }
    }
    let data = serde_json::from_value::<T>(data.into())?;
    Ok((data, files))
}
