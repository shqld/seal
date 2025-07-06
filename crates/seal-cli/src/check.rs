use seal_ty::{checker::Checker, context::TyContext, parse::parse};

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

pub fn check(source: &str) -> CheckResult {
    // Parse the source code
    let parse_result = match parse(source) {
        Ok(result) => result,
        Err(err) => {
            // Handle parse errors
            return CheckResult {
                errors: vec![CheckError {
                    message: format!("Parse error: {:?}", err),
                    start_line: 1,
                    start_column: 1,
                    end_line: 1,
                    end_column: 1,
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
            // Convert errors
            let check_errors = errors
                .into_iter()
                .map(|error| CheckError {
                    message: error.to_string(),
                    start_line: 1,  // TODO: Get real position from AST
                    start_column: 1,
                    end_line: 1,
                    end_column: 1,
                })
                .collect();

            CheckResult {
                errors: check_errors,
            }
        }
    }
}