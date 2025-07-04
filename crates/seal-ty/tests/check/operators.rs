use super::{fail, pass};

pass!(
	arithmetic_operators,
	r#"
        let a = 10;
        let b = 3;
        
        let sum = a + b;
        let diff = a - b;
        let product = a * b;
        let quotient = a / b;
        
        sum satisfies number;
        diff satisfies number;
        product satisfies number;
        quotient satisfies number;
    "#
);

pass!(
	comparison_operators,
	r#"
        let x = 5;
        let y = 10;
        
        let eq = x === y;
        let neq = x !== y;
        let lt = x < y;
        let lte = x <= y;
        let gt = x > y;
        let gte = x >= y;
        
        eq satisfies boolean;
        neq satisfies boolean;
        lt satisfies boolean;
        lte satisfies boolean;
        gt satisfies boolean;
        gte satisfies boolean;
    "#
);

pass!(
	logical_operators,
	r#"
        let a = true;
        let b = false;
        
        let and = a && b;
        let or = a || b;
        let notA = !a;
        
        and satisfies boolean;
        or satisfies boolean;
        notA satisfies boolean;
    "#
);

fail!(
	binary_operator_type_mismatch,
	r#"
        let num = 5;
        let str = "hello";
        let result = num + str;
    "#,
	&["Operator '+' cannot be applied to types 'number' and 'string'."]
);

pass!(
	complex_expression_precedence,
	r#"
        let a = 2;
        let b = 3;
        let c = 4;
        let result = a + b * c;
        result satisfies number;
    "#
);

fail!(
	divide_by_zero_type_check,
	r#"
        let zero = 0;
        let result = 10 / zero;
        result satisfies string;
    "#,
	&["Type 'number' is not assignable to type 'string'."]
);
