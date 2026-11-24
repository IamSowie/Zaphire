//! Engine-independent browser state. Nothing here knows about CEF, so it is
//! fast to test and could drive a different engine later.

pub mod session;
pub mod settings;
pub mod url;

pub use session::{Session, Tab, TabId, Workspace, WorkspaceId};
pub use settings::{Settings, TabLayout, Theme};
