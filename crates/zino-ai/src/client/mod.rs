
pub trait ProviderClient{

    fn from_env() -> Self 
    where 
        Self: Sized;

    fn boxed(self) -> Box<dyn ProviderClient>
    where 
        Self: Sized + 'static,
    {
        Box::new(self)
    }

    fn from_env_boxed<'a>() -> Box<dyn ProviderClient + 'a>
    where 
        Self: Sized + 'a,
    {
        Box::new(Self::from_env())
    }

    fn from_val(input: String) -> Self 
    where 
        Self: Sized;

    fn from_val_boxed<'a>(input: String) -> Box<dyn ProviderClient + 'a>
    where 
        Self: Sized + 'a,
    {
        Box::new(Self::from_val(input))
    }
}


pub trait CompletionClient {
    // fn completion_model(&self, model: &str) -> CompletionModel;
    
}
#[derive(Clone)]
pub enum ProviderValue{
    
}