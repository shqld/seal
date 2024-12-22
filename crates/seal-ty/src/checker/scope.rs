use std::cell::Cell;

use swc_common::SyntaxContext;

pub struct Scope {
	pub ctx: SyntaxContext,
	// TODO: cfg
	pub has_returned: Cell<bool>,
}

impl Scope {
	pub fn new(ctx: SyntaxContext) -> Self {
		Scope {
			ctx,
			has_returned: Cell::new(false),
		}
	}
}
