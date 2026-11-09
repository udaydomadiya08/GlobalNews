pub mod desktop;
pub mod error;
pub mod mobile;
pub mod traits;

pub use desktop::DesktopAdapter;
pub use error::PlatformError;
pub use mobile::MobileAdapter;
pub use traits::PlatformAdapter;
