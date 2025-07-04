use super::pass;

pass!(
	switch_basic,
	r#"
        let x = 1;
        switch (x) {
            case 1:
                let a = "one";
                break;
            case 2:
                let b = "two";
                break;
            default:
                let c = "other";
        }
    "#
);

pass!(
	switch_string,
	r#"
        let status = "pending";
        switch (status) {
            case "pending":
                let message = "Waiting...";
                break;
            case "complete":
                let result = "Done!";
                break;
        }
    "#
);

pass!(
	switch_with_fallthrough,
	r#"
        let grade = "A";
        switch (grade) {
            case "A":
            case "B":
                let message = "Good job";
                break;
            case "C":
                let message2 = "Average";
                break;
            default:
                let message3 = "Needs improvement";
        }
    "#
);

pass!(
	switch_numeric_cases,
	r#"
        let code = 200;
        switch (code) {
            case 200:
                let success = "OK";
                break;
            case 404:
                let notFound = "Not Found";
                break;
            case 500:
                let serverError = "Internal Server Error";
                break;
            default:
                let unknown = "Unknown status";
        }
    "#
);
