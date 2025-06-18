pub mod local;
pub mod s3;
pub mod azure;
pub mod google_cloud;

pub use local::*;
pub use s3::*;
pub use azure::*;
pub use google_cloud::*;