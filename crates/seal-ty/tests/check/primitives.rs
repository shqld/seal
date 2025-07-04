use super::{fail, pass};

pass!(
	unknown_type_basic,
	r#"
        let x: unknown = 42;
        x satisfies unknown;
    "#
);

pass!(
	unknown_accepts_anything,
	r#"
        let x: unknown = "hello";
        let y: unknown = 42;
        let z: unknown = true;
        let obj: unknown = { a: 1 };
    "#
);

fail!(
	unknown_assignment_restriction,
	r#"
        let x: unknown = 42;
        let y: number = x;
    "#,
	&["Type 'unknown' is not assignable to type 'number'."]
);

fail!(
	never_assignment,
	r#"
        let impossible: never = 42;
    "#,
	&["Type 'number' is not assignable to type 'never'."]
);

pass!(
	never_type_in_exhaustive_switch,
	r#"
        let value: "a" | "b" = "a";
        switch (value) {
            case "a":
                let resultA = "A case";
                break;
            case "b":
                let resultB = "B case";
                break;
            default:
                // This should be never type
                let exhaustive: never = value;
        }
    "#
);
