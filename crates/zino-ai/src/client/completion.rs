//use crate::agent::AgentBuilder;
use crate::client::ProviderClient;
use crate::completions::CompletionModel;
/// A provider client with completion capabilities.
/// Clone is required for conversions between client types.
pub trait CompletionClient: ProviderClient + Clone {
    /// The type of CompletionModel used by the client.
    type CompletionModel: CompletionModel;

    fn completion_model(&self, model: &str) -> Self::CompletionModel;
}
