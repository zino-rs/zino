//! Macros for simplifying function registration

/// Macro to create a function with flexible parameters
#[macro_export]
macro_rules! create_function {
    ($name:expr, $description:expr, $schema:expr, $func:expr) => {
        zino_ai::Tool::helpers::json_function($name, $description, $schema, $func)
    };
}

/// Macro to create a function with HashMap parameters
#[macro_export]
macro_rules! create_hashmap_function {
    ($name:expr, $description:expr, $schema:expr, $func:expr) => {
        zino_ai::Tool::helpers::hashmap_function($name, $description, $schema, $func)
    };
}

/// Macro to register a function with flexible parameters
#[macro_export]
macro_rules! register_function {
    ($registry:expr, $name:expr, $description:expr, $schema:expr, $func:expr) => {
        $registry.register(
            $name,
            zino_ai::Tool::helpers::json_function($name, $description, $schema, $func),
        )?;
    };
}
