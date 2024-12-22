pub mod checker;
pub mod context;
mod infer;
mod interner;
mod kind;
mod sema;
mod ty;

pub use kind::TyKind;
pub use ty::Ty;
