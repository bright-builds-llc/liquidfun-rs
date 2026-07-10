use crate::XtaskError;

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn run(_args: &[String]) -> Result<(), XtaskError> {
    Err(XtaskError::not_implemented("inventory"))
}
