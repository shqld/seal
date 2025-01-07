pub mod checker;
pub mod context;
mod infer;
mod interner;
mod kind;
pub mod symbol;
mod ty;

pub use kind::TyKind;
pub use ty::Ty;
