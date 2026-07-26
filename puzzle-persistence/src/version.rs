/// Current save-file schema version.
///
/// **Increment this constant** whenever the `Session` schema changes in a
/// backward-incompatible way, which includes:
/// - Removing a field
/// - Renaming a field
/// - Changing a field's type
/// - Changing the meaning of an existing field's value
pub const SAVE_FORMAT_VERSION: u32 = 1;
