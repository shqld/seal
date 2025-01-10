pub mod checker;
mod interner;
mod kind;
mod ty;

pub use kind::TyKind;
pub use ty::Ty;

// TODO:
pub use crate::kind::infer::{Infer, InferKind};
