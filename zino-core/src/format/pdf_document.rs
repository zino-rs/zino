use crate::{extension::JsonObjectExt, Map};
use printpdf::{
    BuiltinFont, Error, IndirectFontRef, Mm, PdfDocumentReference, PdfLayerReference,
    PdfPageReference,
};

/// PDF document.
pub struct PdfDocument {
    /// Font.
    font: IndirectFontRef,
    /// Font size.
    font_size: f64,
    /// Page width.
    page_width: f64,
    /// Page height.
    page_height: f64,
    /// Left margin.
    margin_left: f64,
    /// Right margin.
    margin_right: f64,
    /// Top margin.
    margin_top: f64,
    /// Bottom margin.
    margin_bottom: f64,
    /// A wrapper for a document.
    document: PdfDocumentReference,
    /// Current page.
    current_page: PdfPageReference,
    /// Current layer.
    current_layer: PdfLayerReference,
    /// Current cursor position.
    current_position: (f64, f64),
    /// Page count.
    page_count: usize,
    /// Layer count.
    layer_count: usize,
}

impl PdfDocument {
    /// Attempts to create a new document with the default settings.
    #[inline]
    pub fn try_new(
        document_title: impl Into<String>,
        page_size: Option<(f64, f64)>,
    ) -> Result<Self, Error> {
        let layer_name = "Page 1, Layer 1";
        let (page_width, page_height) = page_size.unwrap_or((210.0, 297.0));
        let document = printpdf::PdfDocument::empty(document_title);
        let (page_index, layer_index) =
            document.add_page(Mm(page_width), Mm(page_height), layer_name);
        let current_page = document.get_page(page_index);
        let current_layer = current_page.get_layer(layer_index);
        let font = document.add_builtin_font(BuiltinFont::TimesRoman)?;
        Ok(Self {
            font,
            font_size: 12.0,
            page_width,
            page_height,
            margin_left: 12.0,
            margin_right: 12.0,
            margin_top: 20.0,
            margin_bottom: 20.0,
            document,
            current_page,
            current_layer,
            current_position: (12.0, 20.0),
            page_count: 1,
            layer_count: 1,
        })
    }

    /// Sets the page size.
    #[inline]
    pub fn set_page_size(&mut self, page_width: f64, page_height: f64) {
        self.page_width = page_width;
        self.page_height = page_height;
    }

    /// Sets the page margin.
    #[inline]
    pub fn set_margin(
        &mut self,
        margin_left: f64,
        margin_right: f64,
        margin_top: f64,
        margin_bottom: f64,
    ) {
        self.margin_left = margin_left;
        self.margin_right = margin_right;
        self.margin_top = margin_top;
        self.margin_bottom = margin_bottom;
    }

    /// Sets the font size.
    #[inline]
    pub fn set_font_size(&mut self, font_size: f64) {
        self.font_size = font_size;
    }

    /// Adds a new page to the document.
    pub fn add_new_page(&mut self) {
        let document = &self.document;
        let page_count = self.page_count + 1;
        let page_width = Mm(self.page_width);
        let page_height = Mm(self.page_height);
        let layer_name = format!("Page {page_count}, Layer 1");
        let (page_index, layer_index) = document.add_page(page_width, page_height, layer_name);
        let current_page = document.get_page(page_index);
        let current_layer = current_page.get_layer(layer_index);
        self.current_page = current_page;
        self.current_layer = current_layer;
        self.current_position = (self.margin_left, self.margin_top);
        self.page_count = page_count;
    }

    /// Adds a new layer to the current page.
    pub fn add_new_layer(&mut self) {
        let page_count = self.page_count;
        let layer_count = self.layer_count + 1;
        let layer_name = format!("Page {page_count}, Layer {layer_count}");
        self.current_layer = self.current_page.add_layer(layer_name);
        self.current_position = (self.margin_left, self.margin_top);
        self.layer_count = layer_count;
    }

    /// Adds text to the current layer at the position `(x, y)`.
    /// The origin is at the top-left cornner.
    pub fn add_text(&mut self, text: impl ToString, position: (f64, f64)) {
        let font = &self.font;
        let font_size = self.font_size;
        let x = position.0;
        let y = self.page_height - position.1;
        self.current_layer
            .use_text(text.to_string(), font_size, Mm(x), Mm(y), font);
        self.current_position = (x, y + font_size);
    }

    /// Adds a table to the document.
    pub fn add_data_table<const N: usize>(&mut self, data: Vec<&Map>, columns: [&str; N]) {
        let line_height = self.font_size;
        let content_width = self.page_width - self.margin_left - self.margin_right;
        let span_width = content_width / (columns.len() + 1) as f64;
        let (x, mut y) = self.current_position;
        for (index, entry) in data.into_iter().enumerate() {
            let number = index + 1;
            self.add_text(number, (x, y));

            let mut x = x + span_width;
            for col in columns {
                let value = entry.parse_string(col).unwrap_or_default();
                self.add_text(value, (x, y));
                x += span_width;
            }
            y += line_height;
        }
    }

    /// Saves the PDF document to bytes.
    #[inline]
    pub fn save_to_bytes(self) -> Result<Vec<u8>, Error> {
        self.document.save_to_bytes()
    }
}
