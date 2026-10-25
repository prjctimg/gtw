pub mod audio;
pub mod image;
pub mod spike;
pub mod video;

#[allow(dead_code)]
pub fn init() {
    let _ = crate::core::activity::ActivityCenter::new();
}
