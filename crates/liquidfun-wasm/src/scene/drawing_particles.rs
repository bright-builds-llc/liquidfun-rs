//! Drawing Particles and Sparky are experimental native Rust ports of the pinned LiquidFun tests. The playground shows recognizable behavior. Catalog previews are static illustrations.

#[cfg(test)]
mod tests;

use super::BuiltScene;
use crate::session::SessionError;

pub(crate) fn build(_presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    Err(SessionError::SceneConstruction)
}
