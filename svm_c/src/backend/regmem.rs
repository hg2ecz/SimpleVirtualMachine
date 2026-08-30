use crate::{backend_register, model::*};
pub fn emit(p: &Program, l: &Layout, opt: OptLevel) -> Result<String, String> {
    let s = backend_register::emit(p, l, opt)?;
    Ok(s.replace("register-machine target", "register-memory target"))
}
