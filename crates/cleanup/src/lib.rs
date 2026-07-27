pub mod detector;
pub mod duplicate;
pub mod temp;
pub mod docker;

pub use duplicate::detect_duplicates;
pub use docker::detect_stale_docker_volumes;
pub use temp::{detect_temp_files, detect_browser_cache};
