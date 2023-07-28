use mime::{Mime, APPLICATION, AUDIO, IMAGE, JAVASCRIPT, JSON, PDF, TEXT, VIDEO};

/// Returns `ture` if the content can be displayed inline in the browser.
pub(crate) fn displayed_inline(content_type: &Mime) -> bool {
    let mime_type = content_type.type_();
    if matches!(mime_type, TEXT | IMAGE | AUDIO | VIDEO) {
        true
    } else if mime_type == APPLICATION {
        let mime_subtype = content_type.subtype();
        matches!(mime_subtype, JSON | JAVASCRIPT | PDF)
    } else {
        false
    }
}
