use std::collections::HashMap;

pub trait BaseMemory: Send + Sync {
    /*The string keys this memory class will add to chain inputs. */
    fn memory_variables(&self) -> Vec<String>;

    /*Return key-value pairs given the text input to the chain.

        Args:
            inputs: The inputs to the chain.

        Returns:
            A dictionary of key-value pairs.
    */
    fn load_memory_variables(&self, inputs: HashMap<String, String>) -> HashMap<String, String>;

    /*Async return key-value pairs given the text input to the chain.

        Args:
            inputs: The inputs to the chain.

        Returns:
            A dictionary of key-value pairs.
            return await run_in_executor(None, self.load_memory_variables, inputs)
    */
    async fn aload_memory_variables(
        self,
        inputs: HashMap<String, String>,
    ) -> HashMap<String, String>;

    /*Save the context of this chain run to memory.

    Args:
        inputs: The inputs to the chain.
        outputs: The outputs of the chain.
    */
    fn save_context(
        self,
        inputs: HashMap<String, String>,
        outputs: HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /*Async save the context of this chain run to memory.

        Args:
            inputs: The inputs to the chain.
            outputs: The outputs of the chain.
        await run_in_executor(None, self.save_context, inputs, outputs)
    */
    async fn asave_context(
        self,
        inputs: HashMap<String, String>,
        outputs: HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /*Clear memory contents.*/
    fn clear(&self) -> Result<(), Box<dyn std::error::Error>>;

    /*
        Async clear memory contents.

        await run_in_executor(None, self.clear)
    */
    async fn aclear(&self) -> Result<(), Box<dyn std::error::Error>>;
}
