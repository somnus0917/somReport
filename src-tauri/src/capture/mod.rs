pub mod fake;
pub mod x11;

pub use fake::FakeCaptureProvider;
pub use x11::X11CaptureProvider;
