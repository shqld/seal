import type { editor } from "monaco-editor";
import { useCallback, useEffect, useRef, useState } from "react";
import MonacoEditor from "./MonacoEditor";
import type { TypeCheckError, WasmModule } from "./types";
import "bulma/css/bulma.min.css";
// import "./App.css";

const STORAGE_KEY = "seal-typescript-code";

function useTheme() {
	const [isDark, setIsDark] = useState(
		() => window.matchMedia("(prefers-color-scheme: dark)").matches
	);

	useEffect(() => {
		const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
		const handleChange = (e: MediaQueryListEvent) => setIsDark(e.matches);

		mediaQuery.addEventListener("change", handleChange);
		return () => mediaQuery.removeEventListener("change", handleChange);
	}, []);

	useEffect(() => {
		// Apply theme to document
		document.documentElement.setAttribute(
			"data-theme",
			isDark ? "dark" : "light"
		);
	}, [isDark]);

	return isDark;
}

function App() {
	const isDark = useTheme();
	const [code, setCode] = useState(() => {
		// First, try to load from URL parameters
		try {
			const urlParams = new URLSearchParams(window.location.search);
			const codeFromUrl = urlParams.get("code");
			if (codeFromUrl) {
				return decodeURIComponent(atob(codeFromUrl));
			}
		} catch (error) {
			console.warn("Failed to load code from URL:", error);
		}

		// Then try localStorage
		try {
			const savedCode = localStorage.getItem(STORAGE_KEY);
			if (savedCode) {
				return savedCode;
			}
		} catch (error) {
			console.warn("Failed to load code from localStorage:", error);
		}

		// Default code if both URL and localStorage are empty or fail
		return `const x: number = 42;
const y: string = "hello";

function add(a: number, b: number): number {
  return a + b;
}

// This will produce an error
const result: string = add(1, 2);`;
	});
	const [prevCode, setPrevCode] = useState(code);

	const [errors, setErrors] = useState<TypeCheckError[]>([]);
	const [wasmModule, setWasmModule] = useState<WasmModule | null>(null);
	const [isLoading, setIsLoading] = useState(true);
	const [copyStatus, setCopyStatus] = useState<"idle" | "copied" | "error">(
		"idle"
	);

	// Reset copy status when code changes
	if (prevCode !== code) {
		setPrevCode(code);
		setCopyStatus("idle");
	}

	const monacoRef = useRef<typeof import("monaco-editor") | null>(null);
	const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);

	// Load WASM module on component mount
	useEffect(() => {
		const loadWasm = async () => {
			try {
				// Dynamic import of the WASM module
				const wasm = await import("./wasm/seal_cli.js");
				await wasm.default();
				setWasmModule(wasm as unknown as WasmModule);
				setIsLoading(false);
			} catch (error) {
				console.error("Failed to load WASM module:", error);
				setIsLoading(false);
			}
		};

		loadWasm();
	}, []);

	// Update URL when code changes (without page reload)
	useEffect(() => {
		// Don't update URL on initial load
		const urlParams = new URLSearchParams(window.location.search);
		if (urlParams.has("code")) {
			const newUrl = new URL(window.location.href);
			newUrl.searchParams.delete("code");
			window.history.replaceState({}, "", newUrl.toString());
		}
	}, []);

	// Save code to localStorage with debounce
	useEffect(() => {
		const saveTimer = setTimeout(() => {
			try {
				localStorage.setItem(STORAGE_KEY, code);
			} catch (error) {
				console.warn("Failed to save code to localStorage:", error);
			}
		}, 300);

		return () => clearTimeout(saveTimer);
	}, [code]);

	// Type check code whenever it changes
	useEffect(() => {
		if (!wasmModule || !code) return;

		const checkTypes = async () => {
			try {
				const result = wasmModule.type_check(code);
				setErrors(result.errors || []);
				// updateEditorMarkers(result.errors || []);
			} catch (error) {
				console.error("Type checking failed:", error);
				setErrors([
					{
						message: `Internal error: ${error instanceof Error ? error.message : String(error)}`,
						start_line: 1,
						start_column: 1,
						end_line: 1,
						end_column: 1,
					},
				]);
			}
		};

		// Debounce type checking
		const timer = setTimeout(checkTypes, 300);
		return () => clearTimeout(timer);
	}, [code, wasmModule]);

	const updateEditorMarkers = useCallback((errors: TypeCheckError[]) => {
		if (!monacoRef.current || !editorRef.current) return;

		const model = editorRef.current.getModel();
		if (!model) return;

		// Convert errors to Monaco markers
		const markers = errors.map((error) => ({
			startLineNumber: error.start_line,
			startColumn: error.start_column,
			endLineNumber: error.end_line || error.start_line,
			endColumn: error.end_column || error.start_column + 1,
			message: error.message,
			severity: monacoRef.current?.MarkerSeverity.Error ?? 8,
		}));

		// Set markers
		monacoRef.current?.editor.setModelMarkers(
			model,
			"seal-type-checker",
			markers
		);
	}, []);

	const handleEditorDidMount = (
		editor: editor.IStandaloneCodeEditor,
		monaco: typeof import("monaco-editor")
	) => {
		editorRef.current = editor;
		monacoRef.current = monaco;
	};

	useEffect(() => {
		if (errors.length > 0) {
			updateEditorMarkers(errors);
		}
	}, [errors, updateEditorMarkers]);

	const handleErrorClick = (error: TypeCheckError) => {
		if (!editorRef.current) return;

		// Focus editor and jump to error position
		editorRef.current.focus();
		editorRef.current.setPosition({
			lineNumber: error.start_line,
			column: error.start_column,
		});
		editorRef.current.revealPositionInCenter({
			lineNumber: error.start_line,
			column: error.start_column,
		});
	};

	const handleShare = () => {
		try {
			const encodedCode = btoa(encodeURIComponent(code));
			const url = new URL(window.location.href);
			url.searchParams.set("code", encodedCode);

			// Update address bar without page reload
			window.history.replaceState({}, "", url.toString());

			setCopyStatus("copied");
		} catch (error) {
			console.error("Failed to update URL:", error);
			setCopyStatus("error");
		}
	};

	return (
		<div
			className="container is-fluid"
			style={{ padding: "2rem", maxWidth: "1600px", minHeight: "100vh" }}
		>
			<div className="level">
				<div className="level-left">
					<div className="level-item">
						<div>
							<h1 className="title is-2">Seal TypeScript</h1>
							<p className="subtitle is-5">
								A TypeScript written in Rust, running in your browser via
								WebAssembly
							</p>
						</div>
					</div>
				</div>
				<div className="level-right">
					<div className="level-item">
						<button
							type="button"
							className={`button is-primary ${copyStatus === "copied" ? "is-success" : ""} ${copyStatus === "error" ? "is-danger" : ""}`}
							onClick={handleShare}
							disabled={isLoading}
						>
							<span>
								{copyStatus === "copied"
									? "✓ URL Updated!"
									: copyStatus === "error"
										? "✗ Error"
										: "🔗 Share"}
							</span>
						</button>
					</div>
				</div>
			</div>

			{isLoading ? (
				<div className="notification is-info">
					<p>Loading WebAssembly module...</p>
				</div>
			) : (
				<div className="columns">
					<div className="column is-two-thirds">
						<div className="field">
							<div className="label">TypeScript Code</div>
							<div className="box" style={{ padding: 0, overflow: "hidden" }}>
								<MonacoEditor
									height="600px"
									value={code}
									onChange={(value) => setCode(value || "")}
									onMount={handleEditorDidMount}
									theme={isDark ? "vs-dark" : "vs"}
								/>
							</div>
						</div>
					</div>

					<div className="column is-one-third">
						<div className="field">
							<div className="label">Type Checking Results</div>
							{errors.length === 0 ? (
								<div className="notification is-success">
									<span>No type errors found!</span>
								</div>
							) : (
								<div>
									<div className="notification is-danger">
										<span>
											{errors.length} error{errors.length > 1 ? "s" : ""} found
										</span>
									</div>
									<div
										className="error-list"
										style={{ maxHeight: "500px", overflowY: "auto" }}
									>
										{errors.map((error, index) => (
											<button
												type="button"
												key={`error-${error.start_line}-${error.start_column}-${index}`}
												className="message is-danger error-item"
												style={{
													marginBottom: "0.5rem",
													cursor: "pointer",
													width: "100%",
													border: "none",
													textAlign: "left",
												}}
												onClick={() => handleErrorClick(error)}
											>
												<div
													className="message-body"
													style={{ padding: "0.75rem" }}
												>
													<p
														style={{
															fontFamily: "monospace",
															fontSize: "12px",
															marginBottom: "0.25rem",
														}}
													>
														<strong>
															Line {error.start_line}:{error.start_column}
														</strong>
													</p>
													<p style={{ fontSize: "14px" }}>{error.message}</p>
												</div>
											</button>
										))}
									</div>
								</div>
							)}
						</div>
					</div>
				</div>
			)}

			<footer
				className="footer"
				style={{ marginTop: "3rem", padding: "2rem 0" }}
			>
				<div className="content has-text-centered">
					<p>
						<strong>Seal</strong> - A TypeScript implementation in Rust
					</p>
				</div>
			</footer>
		</div>
	);
}

export default App;
