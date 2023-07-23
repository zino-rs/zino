/// Masks text with masking options.
pub(crate) fn mask_text(text: &str, num_prefix_chars: usize, num_suffix_chars: usize) -> String {
    let length = text.len();
    let suffix_index = if length > num_suffix_chars {
        length - num_suffix_chars
    } else {
        0
    };

    let mut masked_text = String::with_capacity(length);
    for (i, c) in text.chars().enumerate() {
        if i < num_prefix_chars || i >= suffix_index {
            masked_text.push(c);
        } else {
            masked_text.push('*');
        }
    }
    masked_text
}
