use super::engine::PlaybackEngine;

#[cfg(target_os = "macos")]
pub fn create_engine() -> Box<dyn PlaybackEngine + Send> {
    Box::new(super::av_proxy_engine::AvProxyEngine::new())
}

#[cfg(not(target_os = "macos"))]
pub fn create_engine() -> Box<dyn PlaybackEngine + Send> {
    Box::new(super::rodio_engine::RodioEngine::new())
}
