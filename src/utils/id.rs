/// Generates a unique identifier as an optional `String`.
///
/// This function creates a 30-character long unique identifier composed of
/// uppercase English letters (`A-Z`) and numbers (`0-9`) using the `nanoid`
/// library. The generated identifier is securely random and designed to be
/// collision-resistant.
///
/// # Returns
/// - `Some(String)`: A generated identifier as a `String` if successful.
/// - `None`: This function is designed to always return `Some`, but this is
///   wrapped in an `Option` for potential extension or compatibility with
///   other code.
///
/// # Examples
/// ```
/// use ig_client::utils::id::get_id;
/// let unique_id = get_id();
/// if let Some(id) = unique_id {
///     println!("Generated ID: {}", id);
/// }
/// ```
///
/// # Dependencies
/// - This function relies on the `nanoid` crate, which must be added to your
///   project dependencies in `Cargo.toml`:
///
/// ```toml
/// [dependencies]
/// nanoid = "0.4"
/// ```
pub fn get_id() -> Option<String> {
    let alphabet: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
    Some(nanoid::nanoid!(30, &alphabet))
}
