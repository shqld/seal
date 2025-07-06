export interface TypeCheckError {
	message: string;
	start_line: number;
	start_column: number;
	end_line?: number;
	end_column?: number;
}

export interface TypeCheckResult {
	errors: TypeCheckError[];
}

export interface WasmModule {
	type_check: (code: string) => TypeCheckResult;
	default: () => Promise<void>;
}
