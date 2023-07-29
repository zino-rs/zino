//! HTTP file uploading and downloading.

use crate::{
    crypto,
    encoding::{base64, hex},
    error::Error,
};
use bytes::Bytes;
use mime::Mime;
use multer::{Field, Multipart};
use std::{
    convert::AsRef,
    fs::{self, OpenOptions},
    io::{self, Write},
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
}

impl NamedFile {
    /// Creates a new instance with the specific file name.
    #[inline]
    pub fn new(file_name: impl Into<String>) -> Self {
        Self {
            field_name: None,
            file_name: Some(file_name.into()),
            content_type: None,
            bytes: Bytes::new(),
        }
    }

    /// Sets the field name.
    #[inline]
    pub fn set_field_name(&mut self, field_name: impl Into<String>) {
        self.field_name = Some(field_name.into());
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

    /// Returns the bytes.
    #[inline]
    pub fn bytes(&self) -> Bytes {
        self.bytes.clone()
    }

    /// Returns the checksum for the file.
    #[inline]
    pub fn checksum(&self) -> Bytes {
        let checksum = crypto::sha256(self.as_ref());
        Vec::from(checksum).into()
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
    pub fn read_from_local<P: AsRef<Path>>(&mut self, path: P) -> Result<(), io::Error> {
        let bytes = fs::read(path)?;
        self.bytes = bytes.into();
        Ok(())
    }

    /// Writes the bytes into path.
    #[inline]
    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), io::Error> {
        fs::write(path, self.as_ref())
    }

    /// Appends the bytes into path.
    #[inline]
    pub fn append<P: AsRef<Path>>(&self, path: P) -> Result<(), io::Error> {
        let mut file = OpenOptions::new().write(true).append(true).open(path)?;
        file.write_all(self.as_ref())
    }

    /// Encrypts the file with a key.
    #[inline]
    pub fn encrypt_with(&mut self, key: impl AsRef<[u8]>) -> Result<(), Error> {
        let suffix = ".encrypted";
        let bytes = crypto::encrypt(self.as_ref(), key.as_ref())?;
        if let Some(ref mut file_name) = self.file_name && !file_name.ends_with(suffix) {
            file_name.push_str(suffix);
        }
        self.bytes = bytes.into();
        Ok(())
    }

    /// Decrypts the file with a key.
    #[inline]
    pub fn decrypt_with(&mut self, key: impl AsRef<[u8]>) -> Result<(), Error> {
        let suffix = ".encrypted";
        let bytes = crypto::decrypt(self.as_ref(), key.as_ref())?;
        if let Some(ref mut file_name) = self.file_name && file_name.ends_with(suffix) {
            file_name.truncate(file_name.len() - suffix.len());
        }
        self.bytes = bytes.into();
        Ok(())
    }

    /// Attempts to create an instance from reading a local file.
    pub fn try_from_local<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let path = path.as_ref();
        let bytes = fs::read(path)?;
        let file_name = path.file_name().map(|s| s.to_string_lossy().into_owned());
        let content_type = file_name.as_ref().and_then(|s| {
            let file_name = s.strip_suffix(".encrypted").unwrap_or(s);
            mime_guess::from_path(file_name).first()
        });
        Ok(Self {
            field_name: None,
            file_name,
            content_type,
            bytes: bytes.into(),
        })
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
        })
    }

    /// Attempts to create a file in a multipart stream.
    pub async fn try_from_multipart(mut multipart: Multipart<'_>) -> Result<Self, multer::Error> {
        while let Some(field) = multipart.next_field().await? {
            if field.file_name().is_some() {
                let file = NamedFile::try_from_multipart_field(field).await?;
                return Ok(file);
            }
        }
        Err(multer::Error::IncompleteFieldData { field_name: None })
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
