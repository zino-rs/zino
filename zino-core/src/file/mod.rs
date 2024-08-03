//! HTTP file uploading and downloading.

use crate::{
    application::http_client,
    crypto,
    encoding::{base64, hex},
    error::Error,
    extension::{JsonObjectExt, JsonValueExt},
    json,
    trace::TraceContext,
    warn, JsonValue, Map,
};
use bytes::Bytes;
use etag::EntityTag;
use md5::{Digest, Md5};
use mime_guess::Mime;
use multer::{Field, Multipart};
use reqwest::{
    multipart::{Form, Part},
    Response,
};
use std::{
    borrow::Cow,
    fs::{self, File, OpenOptions},
    io::{self, ErrorKind, Read, Write},
    path::Path,
};

/// A file with an associated name.
#[derive(Debug, Clone, Default)]
pub struct NamedFile {
    /// Field name.
    field_name: Option<String>,
    /// File name.
    file_name: Option<String>,
    /// Content type.
    content_type: Option<Mime>,
    /// Bytes.
    bytes: Bytes,
    /// Extra attributes.
    extra: Map,
}

impl NamedFile {
    /// Creates a new instance with the specific file name.
    pub fn new(file_name: impl Into<String>) -> Self {
        let file_name = file_name.into();
        let content_type = mime_guess::from_path(&file_name).first();
        Self {
            field_name: None,
            file_name: Some(file_name),
            content_type,
            bytes: Bytes::new(),
            extra: Map::new(),
        }
    }

    /// Sets the field name.
    #[inline]
    pub fn set_field_name(&mut self, field_name: impl Into<String>) {
        self.field_name = Some(field_name.into());
    }

    /// Sets the file name.
    #[inline]
    pub fn set_file_name(&mut self, file_name: impl Into<String>) {
        self.file_name = Some(file_name.into());
    }

    /// Sets the content type.
    #[inline]
    pub fn set_content_type(&mut self, content_type: Mime) {
        self.content_type = Some(content_type);
    }

    /// Sets the bytes.
    #[inline]
    pub fn set_bytes(&mut self, bytes: impl Into<Bytes>) {
        self.bytes = bytes.into();
    }

    /// Sets the extra attribute.
    ///
    /// # Note
    ///
    /// Currently, we support the following built-in attributes:
    /// `checksum` | `chunk_number` | `chunk_size` | `total_chunks`.
    #[inline]
    pub fn set_extra_attribute(&mut self, key: &str, value: impl Into<JsonValue>) {
        self.extra.upsert(key, value);
    }

    /// Appends the extra attributes.
    #[inline]
    pub fn append_extra_attributes(&mut self, attrs: &mut Map) {
        self.extra.append(attrs);
    }

    /// Returns the field name corresponding to the file.
    #[inline]
    pub fn field_name(&self) -> Option<&str> {
        self.field_name.as_deref()
    }

    /// Returns the file name.
    #[inline]
    pub fn file_name(&self) -> Option<&str> {
        self.file_name.as_deref()
    }

    /// Returns the content type.
    #[inline]
    pub fn content_type(&self) -> Option<&Mime> {
        self.content_type.as_ref()
    }

    /// Returns the file size.
    #[inline]
    pub fn file_size(&self) -> u64 {
        self.bytes.len().try_into().unwrap_or_default()
    }

    /// Returns the bytes.
    #[inline]
    pub fn bytes(&self) -> Bytes {
        self.bytes.clone()
    }

    /// Returns a reference to the extra attributes.
    #[inline]
    pub fn extra(&self) -> &Map {
        &self.extra
    }

    /// Returns the chunk number for the file.
    #[inline]
    pub fn chunk_number(&self) -> Option<usize> {
        self.extra.parse_usize("chunk_number")?.ok()
    }

    /// Returns the chunk size for the file.
    #[inline]
    pub fn chunk_size(&self) -> Option<usize> {
        self.extra.parse_usize("chunk_size")?.ok()
    }

    /// Returns the total number of file chunks.
    #[inline]
    pub fn total_chunks(&self) -> Option<usize> {
        self.extra.parse_usize("total_chunks")?.ok()
    }

    /// Returns the checksum for the file.
    ///
    /// # Note
    ///
    /// If you would like to obtain a hex-formated string,
    /// you can use the `LowerHex` or `UpperHex` implementations for `Bytes`.
    #[inline]
    pub fn checksum(&self) -> Bytes {
        let checksum = crypto::checksum(self.as_ref());
        Vec::from(checksum).into()
    }

    /// Returns the ETag for the file.
    #[inline]
    pub fn etag(&self) -> EntityTag {
        EntityTag::from_data(self.as_ref())
    }

    /// Returns the content MD5.
    pub fn content_md5(&self) -> String {
        let mut hasher = Md5::new();
        hasher.update(self.as_ref());
        base64::encode(hasher.finalize())
    }

    /// Returns the hex representation of the file bytes.
    #[inline]
    pub fn to_hex_string(&self) -> String {
        hex::encode(self.as_ref())
    }

    /// Returns the base64 representation of the file bytes.
    #[inline]
    pub fn to_base64_string(&self) -> String {
        base64::encode(self.as_ref())
    }

    /// Reads the string and sets the bytes.
    #[inline]
    pub fn read_string(&mut self, data: String) -> Result<(), Error> {
        self.bytes = data.into();
        Ok(())
    }

    /// Reads the hex string and sets the bytes.
    #[inline]
    pub fn read_hex_string(&mut self, data: &str) -> Result<(), Error> {
        let bytes = hex::decode(data)?;
        self.bytes = bytes.into();
        Ok(())
    }

    /// Reads the base64 string and sets the bytes.
    #[inline]
    pub fn read_base64_string(&mut self, data: &str) -> Result<(), Error> {
        let bytes = base64::decode(data)?;
        self.bytes = bytes.into();
        Ok(())
    }

    /// Reads the entire contents of a local file and sets the bytes.
    #[inline]
    pub fn read_from_local(&mut self, path: impl AsRef<Path>) -> Result<(), io::Error> {
        fn inner(file: &mut NamedFile, path: &Path) -> Result<(), io::Error> {
            let bytes = fs::read(path)?;
            file.bytes = bytes.into();
            Ok(())
        }
        inner(self, path.as_ref())
    }

    /// Writes the bytes into a file at the path.
    /// If the extra attributes contain a `chunk_number` value,
    /// a `.{chunk_number}.part` suffix will be adjoined to the path.
    pub fn write(&self, path: impl AsRef<Path>) -> Result<(), io::Error> {
        fn inner(file: &NamedFile, path: &Path) -> Result<(), io::Error> {
            let bytes = file.as_ref();
            if let Some(chunk_number) = file.chunk_number() {
                let chunk_path = path.join(format!(".{chunk_number}.part"));
                fs::write(chunk_path, bytes)
            } else {
                fs::write(path, bytes)
            }
        }
        inner(self, path.as_ref())
    }

    /// Appends the bytes into a file at the path.
    #[inline]
    pub fn append(&self, path: impl AsRef<Path>) -> Result<(), io::Error> {
        fn inner(file: &NamedFile, path: &Path) -> Result<(), io::Error> {
            OpenOptions::new()
                .append(true)
                .open(path)?
                .write_all(file.as_ref())
        }
        inner(self, path.as_ref())
    }

    /// Encrypts the file with a key.
    pub fn encrypt_with(&mut self, key: impl AsRef<[u8]>) -> Result<(), Error> {
        fn inner(file: &mut NamedFile, key: &[u8]) -> Result<(), Error> {
            let suffix = ".encrypted";
            let bytes = crypto::encrypt(file.as_ref(), key)?;
            if let Some(ref mut file_name) = file.file_name {
                if !file_name.ends_with(suffix) {
                    file_name.push_str(suffix);
                }
            }
            file.bytes = bytes.into();
            Ok(())
        }
        inner(self, key.as_ref())
    }

    /// Decrypts the file with a key.
    pub fn decrypt_with(&mut self, key: impl AsRef<[u8]>) -> Result<(), Error> {
        fn inner(file: &mut NamedFile, key: &[u8]) -> Result<(), Error> {
            let suffix = ".encrypted";
            let bytes = crypto::decrypt(file.as_ref(), key)?;
            if let Some(ref mut file_name) = file.file_name {
                if file_name.ends_with(suffix) {
                    file_name.truncate(file_name.len() - suffix.len());
                }
            }
            file.bytes = bytes.into();
            Ok(())
        }
        inner(self, key.as_ref())
    }

    /// Renames the stem portion of the file name.
    #[inline]
    pub fn rename_file_stem(&mut self, file_stem: &str) -> Result<(), Error> {
        let file_name = if let Some(ext) = self
            .file_name
            .as_ref()
            .and_then(|s| Path::new(s).extension())
        {
            let ext = ext.to_string_lossy();
            format!("{file_stem}.{ext}")
        } else {
            file_stem.to_owned()
        };
        self.file_name = Some(file_name);
        Ok(())
    }

    /// Splits the file into chunks with the `chunk_size`.
    /// The file name of chunks will end with `.{chunk_number}.part`
    /// and the extra attributes will contain the `chunk_number` and `total_chunks`.
    pub fn split_chunks(&self, chunk_size: usize) -> Vec<Self> {
        let file_name = self.file_name().unwrap_or_default();
        let chunks = self.bytes.chunks(chunk_size);
        let total_chunks = chunks.len();
        chunks
            .enumerate()
            .map(|(index, chunk)| {
                let mut file = Self::default();
                file.set_file_name(format!("{file_name}.{index}.part"));
                file.set_bytes(chunk.to_vec());
                file.set_extra_attribute("chunk_number", index);
                file.set_extra_attribute("chunk_size", file.file_size());
                file.set_extra_attribute("total_chunks", total_chunks);
                file
            })
            .collect()
    }

    /// Attempts to concat the file chunks into a whole.
    /// The path should not contain the `.{chunk_number}.part` suffix.
    pub fn try_concat_chunks(
        path: impl AsRef<Path>,
        total_chunks: usize,
    ) -> Result<Self, io::Error> {
        fn inner(path: &Path, total_chunks: usize) -> Result<NamedFile, io::Error> {
            let file_name = path.file_name().map(|s| s.to_string_lossy().into_owned());
            let mut chunk_paths = Vec::with_capacity(total_chunks);
            for index in 0..total_chunks {
                let chunk_path = path.join(format!(".{index}.part"));
                if chunk_path.try_exists()? {
                    chunk_paths.push(chunk_path);
                } else {
                    let file_name = file_name.unwrap_or_default();
                    let message = format!("chunk file `{file_name}.{index}.part` does not exist");
                    return Err(io::Error::new(ErrorKind::NotFound, message));
                }
            }

            let content_type = file_name.as_ref().and_then(|s| {
                let file_name = s.strip_suffix(".encrypted").unwrap_or(s);
                mime_guess::from_path(file_name).first()
            });
            let mut buffer = Vec::new();
            for chunk_path in &chunk_paths {
                File::open(chunk_path)?.read_to_end(&mut buffer)?;
            }
            for chunk_path in chunk_paths {
                if let Err(err) = fs::remove_file(chunk_path) {
                    warn!("fail to remove the file chunk: {}", err);
                }
            }
            Ok(NamedFile {
                field_name: None,
                file_name,
                content_type,
                bytes: buffer.into(),
                extra: Map::new(),
            })
        }
        inner(path.as_ref(), total_chunks)
    }

    /// Attempts to create an instance from reading a local file.
    pub fn try_from_local(path: impl AsRef<Path>) -> Result<Self, io::Error> {
        fn inner(path: &Path) -> Result<NamedFile, io::Error> {
            let bytes = fs::read(path)?;
            let file_name = path.file_name().map(|s| s.to_string_lossy().into_owned());
            let content_type = file_name.as_ref().and_then(|s| {
                let file_name = s.strip_suffix(".encrypted").unwrap_or(s);
                mime_guess::from_path(file_name).first()
            });
            Ok(NamedFile {
                field_name: None,
                file_name,
                content_type,
                bytes: bytes.into(),
                extra: Map::new(),
            })
        }
        inner(path.as_ref())
    }

    /// Attempts to create an instance from a field in a multipart stream.
    pub async fn try_from_multipart_field(field: Field<'_>) -> Result<Self, multer::Error> {
        let field_name = field.name().map(|s| s.to_owned());
        let file_name = field.file_name().map(|s| s.to_owned());
        let content_type = field.content_type().cloned().or_else(|| {
            file_name
                .as_ref()
                .and_then(|s| mime_guess::from_path(s).first())
        });
        let bytes = field.bytes().await?;
        Ok(Self {
            field_name,
            file_name,
            content_type,
            bytes,
            extra: Map::new(),
        })
    }

    /// Attempts to create a file in a multipart stream.
    /// If the extra attributes contain a `chunk_size` or `checksum` value,
    /// the file integrity will be checked.
    pub async fn try_from_multipart(mut multipart: Multipart<'_>) -> Result<Self, multer::Error> {
        let mut extracted_file = None;
        let mut extra = Map::new();
        while let Some(field) = multipart.next_field().await? {
            if field.file_name().is_some() && extracted_file.is_none() {
                let file = NamedFile::try_from_multipart_field(field).await?;
                extracted_file = Some(file);
            } else if let Some(name) = field.name() {
                let key = name.to_owned();
                let value = field.text().await?;
                extra.upsert(key, value);
            }
        }
        if let Some(mut file) = extracted_file {
            if let Some(Ok(chunk_size)) = extra.parse_u64("chunk_size") {
                if file.file_size() != chunk_size {
                    return Err(multer::Error::IncompleteStream);
                }
            }
            if let Some(checksum) = extra.get_str("checksum") {
                let integrity = format!("{:x}", file.checksum());
                if !integrity.eq_ignore_ascii_case(checksum) {
                    return Err(multer::Error::IncompleteStream);
                }
            }
            file.append_extra_attributes(&mut extra);
            Ok(file)
        } else {
            Err(multer::Error::IncompleteFieldData { field_name: None })
        }
    }

    /// Attempts to create a list of files in a multipart stream.
    pub async fn try_collect_from_multipart(
        mut multipart: Multipart<'_>,
    ) -> Result<Vec<Self>, multer::Error> {
        let mut files = Vec::new();
        while let Some(field) = multipart.next_field().await? {
            if field.file_name().is_some() {
                let file = NamedFile::try_from_multipart_field(field).await?;
                files.push(file);
            }
        }
        Ok(files)
    }

    /// Downloads a file from the URL.
    pub async fn download_from(url: &str, options: Option<&Map>) -> Result<Self, Error> {
        let mut trace_context = TraceContext::new();
        let span_id = trace_context.span_id();
        trace_context
            .trace_state_mut()
            .push("zino", format!("{span_id:x}"));

        let response = http_client::request_builder(url, options)?
            .header("traceparent", trace_context.traceparent())
            .header("tracestate", trace_context.tracestate())
            .send()
            .await?
            .error_for_status()?;
        let content_type = response
            .headers()
            .get("content-type")
            .map(|v| v.to_str())
            .transpose()?
            .map(|s| s.parse())
            .transpose()?;
        let bytes = response.bytes().await?;
        Ok(Self {
            field_name: None,
            file_name: None,
            content_type,
            bytes,
            extra: Map::new(),
        })
    }

    /// Uploads the file to the URL.
    pub async fn upload_to(&self, url: &str, options: Option<&Map>) -> Result<Response, Error> {
        let mut trace_context = TraceContext::new();
        let span_id = trace_context.span_id();
        trace_context
            .trace_state_mut()
            .push("zino", format!("{span_id:x}"));

        let mut form = Form::new();
        for (key, value) in self.extra() {
            form = form.text(key.to_owned(), value.to_string_unquoted());
        }

        let field_name = self
            .field_name()
            .map(|s| Cow::Owned(s.to_owned()))
            .unwrap_or_else(|| Cow::Borrowed("file"));
        let mut part = Part::stream_with_length(self.bytes(), self.file_size());
        if let Some(file_name) = self.file_name() {
            part = part.file_name(file_name.to_owned());
        }
        if let Some(content_type) = self.content_type() {
            part = part.mime_str(content_type.essence_str())?;
        }
        form = form.part(field_name, part).percent_encode_noop();

        let request_builder = if options.is_some() {
            http_client::request_builder(url, options)?
        } else {
            let options = json!({
                "method": "POST",
                "data_type": "multipart",
            });
            http_client::request_builder(url, options.as_object())?
        };
        request_builder
            .header("traceparent", trace_context.traceparent())
            .header("tracestate", trace_context.tracestate())
            .multipart(form)
            .send()
            .await
            .map_err(Error::from)
    }
}

impl AsRef<[u8]> for NamedFile {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

impl From<NamedFile> for Bytes {
    #[inline]
    fn from(file: NamedFile) -> Self {
        file.bytes
    }
}

impl<'a> From<&'a NamedFile> for Bytes {
    #[inline]
    fn from(file: &'a NamedFile) -> Self {
        file.bytes()
    }
}

#[cfg(feature = "accessor")]
impl From<NamedFile> for opendal::Buffer {
    #[inline]
    fn from(file: NamedFile) -> Self {
        file.bytes.into()
    }
}

#[cfg(feature = "accessor")]
impl<'a> From<&'a NamedFile> for opendal::Buffer {
    #[inline]
    fn from(file: &'a NamedFile) -> Self {
        file.bytes().into()
    }
}
