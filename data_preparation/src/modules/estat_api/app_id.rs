use std::env::{self, VarError};

/// Retrieves the `ESTAT_APP_ID` environment variable.
///
/// This function attempts to fetch the value of the `ESTAT_APP_ID`
/// environment variable and returns it as a `Result`. If the variable
/// is not set or cannot be accessed, a `VarError` is returned.
pub fn retrieve_estat_app_id() -> Result<String, VarError> {
    env::var("ESTAT_APP_ID")
}
