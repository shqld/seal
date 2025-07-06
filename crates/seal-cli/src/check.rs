use seal_ty::{checker::Checker, context::TyContext, parse::parse};
use swc_common::{SourceMap, Span, Spanned};

pub struct CheckResult {
	pub errors: Vec<CheckError>,
}

pub struct CheckError {
	pub message: String,
	pub start_line: u32,
	pub start_column: u32,
	pub end_line: u32,
	pub end_column: u32,
}

fn span_to_line_col(source_map: &SourceMap, span: Span) -> (u32, u32, u32, u32) {
	// Try to lookup char positions, but handle failures gracefully
	let start_loc = source_map.lookup_char_pos(span.lo);
	let end_loc = source_map.lookup_char_pos(span.hi);

	// Validate the results
	let start_line = start_loc.line.max(1) as u32;
	let start_col = (start_loc.col_display as u32 + 1).max(1);
	let end_line = end_loc.line.max(start_loc.line) as u32;
	let end_col = (end_loc.col_display as u32 + 1).max(start_col + 1);

	(start_line, start_col, end_line, end_col)
}

pub fn check(source: &str) -> CheckResult {
	// Parse the source code
	let parse_result = match parse(source) {
		Ok(result) => result,
		Err(err) => {
			// Handle parse errors - extract position info from SWC error
			let error_msg = format!("Parse error: {:?}", err);

			// Try to extract span information from the parse error
			// SWC parse errors often contain span information
			let span = err.span();
			let (start_line, start_column, end_line, end_column) = if !span.is_dummy() {
				// Create a temporary source map for the current source
				let temp_source_map = swc_common::SourceMap::new(Default::default());
				let _temp_source_file = temp_source_map.new_source_file(
					swc_common::FileName::Custom("temp.ts".to_owned()).into(),
					source.into(),
				);
				span_to_line_col(&temp_source_map, span)
			} else {
				// Fallback to beginning of file if no span info
				(1, 1, 1, 1)
			};

			return CheckResult {
				errors: vec![CheckError {
					message: error_msg,
					start_line,
					start_column,
					end_line,
					end_column,
				}],
			};
		}
	};

	// Type check
	let tcx = TyContext::new();
	let checker = Checker::new(&tcx);

	match checker.check(&parse_result.program) {
		Ok(()) => CheckResult { errors: vec![] },
		Err(errors) => {
			// Convert errors with real position information
			let check_errors = errors
				.into_iter()
				.map(|error| {
					let (start_line, start_column, end_line, end_column) =
						span_to_line_col(&parse_result.source_map, error.span);

					CheckError {
						message: error.to_string(),
						start_line,
						start_column,
						end_line,
						end_column,
					}
				})
				.collect();

			CheckResult {
				errors: check_errors,
			}
		}
	}
}
