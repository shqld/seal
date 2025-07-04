use super::{fail, pass};

pass!(
	try_catch_basic,
	r#"
        try {
            let x = 42;
            x satisfies number;
        } catch (e) {
            let error = "Error occurred";
            error satisfies string;
        }
    "#
);

pass!(
	try_catch_finally,
	r#"
        try {
            let x = 42;
        } catch (e) {
            let error = "Error";
        } finally {
            let cleanup = true;
            cleanup satisfies boolean;
        }
    "#
);

pass!(
	throw_statement,
	r#"
        function throwError() {
            throw "An error occurred";
        }
        
        throwError satisfies () => void;
    "#
);

pass!(
	throw_object,
	r#"
        function throwError() {
            throw { message: "Error", code: 500 };
        }
        
        throwError satisfies () => void;
    "#
);

pass!(
	catch_parameter_unknown,
	r#"
        try {
            let x = 42;
        } catch (e) {
            e satisfies unknown;
        }
    "#
);

fail!(
	catch_parameter_with_type_annotation,
	r#"
        try {
            let x = 42;
        } catch (e: string) {
            let error = e;
        }
    "#,
	&["Catch clause parameter cannot have a type annotation."]
);

pass!(
	catch_parameter_usage,
	r#"
        try {
            throw "error";
        } catch (error) {
            let msg = error;
            msg satisfies unknown;
        }
    "#
);

pass!(
	try_catch_nested,
	r#"
        try {
            try {
                let x = 42;
                throw "inner error";
            } catch (innerError) {
                innerError satisfies unknown;
                throw "outer error";
            }
        } catch (outerError) {
            outerError satisfies unknown;
        }
    "#
);

pass!(
	try_catch_finally_complex,
	r#"
        let resource = "acquired";
        try {
            let data = "processing";
            throw data;
        } catch (error) {
            error satisfies unknown;
            let errorMessage = "handled";
        } finally {
            resource = "released";
        }
    "#
);
