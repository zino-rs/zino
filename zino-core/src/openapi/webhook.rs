use crate::response::WebHook;

/// Gets a webhook with the specific name.
#[inline]
pub(crate) fn get_webhook(name: &str) -> Option<&'static WebHook> {
    super::WEBHOOK_DEFINITIONS
        .get()
        .and_then(|webhooks| webhooks.get(name))
}
