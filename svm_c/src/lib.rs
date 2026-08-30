pub mod backend;
pub mod common;
pub mod optimized;
pub mod unopt;

// Compatibility re-exports keep the backend implementation focused on code
// generation while the on-disk layout clearly separates common and target code.
pub use backend::accumulator as backend_accumulator;
pub use backend::belt as backend_belt;
pub use backend::loadstore as backend_loadstore;
pub use backend::memory2memory as backend_memory2memory;
pub use backend::memreg as backend_memreg;
pub use backend::register as backend_register;
pub use backend::regmem as backend_regmem;
pub use backend::stack as backend_stack;
pub use backend::tta as backend_tta;
pub use common::frontend;
pub use common::layout;
pub use common::model;
pub use common::semantic;
pub use optimized::optimizer as optimize;
