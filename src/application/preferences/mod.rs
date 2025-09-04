mod preferences_manager;
mod version1;
mod version2;

mod version_next; // Copy the next version from this file.

// PreferencesManager responsible for managing preferences versions, saving and loading:
pub use preferences_manager::PreferencesManager;
// Latest version for public usage:
pub use version2::*;
