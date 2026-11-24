//! Engine-independent browser state. Nothing here knows about CEF, so it is
//! fast to test and could drive a different engine later.

pub mod extensions;
pub mod library;
pub mod session;
pub mod settings;
pub mod url;

pub use extensions::ExtensionInfo;
pub use library::{Bookmark, Library, Visit};
pub use session::{Session, Tab, TabId, Workspace, WorkspaceId};
pub use settings::{Background, Density, Settings, TabLayout, Theme, ThemeMode};
