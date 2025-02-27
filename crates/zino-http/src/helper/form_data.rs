use mime_guess::mime::{APPLICATION, JSON};
use multer::Multipart;
use serde::de::DeserializeOwned;
use zino_core::{Map, error::Error, extension::JsonObjectExt};
use zino_storage::NamedFile;

/// Parses the multipart form as an instance of `T` with the `name` and a list of files.
pub(crate) async fn parse_form<T: DeserializeOwned>(
    mut multipart: Multipart<'_>,
    name: &str,
) -> Result<(Option<T>, Vec<NamedFile>), Error> {
    let mut data = None;
    let mut files = Vec::new();
    while let Some(field) = multipart.next_field().await? {
        if field.file_name().is_some() {
            let file = NamedFile::try_from_multipart_field(field).await?;
            files.push(file);
        } else if field.name().is_some_and(|s| s == name) {
            data = Some(field.json().await?);
        }
    }
    Ok((data, files))
}

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
            if field
                .content_type()
                .is_some_and(|m| m.type_() == APPLICATION && m.subtype() == JSON)
            {
                data.upsert(key, field.json::<Map>().await?);
            } else {
                data.upsert(key, field.text().await?);
            }
        }
    }

    let data = serde_json::from_value::<T>(data.into())?;
    Ok((data, files))
}
