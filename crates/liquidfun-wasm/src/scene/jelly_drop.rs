//! Unimplemented Jelly Drop scene. A later plan owns the persistent world.

use super::BuiltScene;
use crate::session::SessionError;

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    let _ = presets;
    Err(SessionError::SceneUnimplemented)
}
