use super::pass;

pass!(
	while_loop_basic,
	r#"
        let i = 0;
        while (true) {
            i = 42;
        }
        i satisfies number;
    "#
);

pass!(
	do_while_loop_basic,
	r#"
        let i = 0;
        do {
            i = i + 1;
        } while (i < 10);
        i satisfies number;
    "#
);

pass!(
	for_loop_basic,
	r#"
        for (let i = 0; i < 10; i = i + 1) {
            let x = i * 2;
            x satisfies number;
        }
    "#
);

pass!(
	for_loop_no_init,
	r#"
        let i = 0;
        for (; i < 10; i = i + 1) {
            let x = i * 2;
            x satisfies number;
        }
    "#
);

pass!(
	for_loop_empty,
	r#"
        for (;;) {
            break;
        }
    "#
);

pass!(
	for_loop_with_complex_init,
	r#"
        for (let i = 0; i < 10; i = i + 1) {
            let value = i * 2;
            value satisfies number;
        }
    "#
);

pass!(
	nested_loops,
	r#"
        for (let i = 0; i < 3; i = i + 1) {
            for (let j = 0; j < 3; j = j + 1) {
                let product = i * j;
                product satisfies number;
            }
        }
    "#
);

pass!(
	while_loop_with_break_continue,
	r#"
        let count = 0;
        while (count < 10) {
            if (count === 5) {
                count = count + 1;
                continue;
            }
            if (count === 8) {
                break;
            }
            count = count + 1;
        }
    "#
);
