use std::result::Result;

pub fn extract_option_result<T, E>(
    input: Option<Result<T, E>>,
) -> Result<Option<T>, E> {
    match input {
        None => Ok(None),
        Some(x) => Ok(Some(x?)),
    }
}
