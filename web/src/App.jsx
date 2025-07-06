import { useState, useEffect, useRef } from 'react'
import MonacoEditor from './MonacoEditor'
import 'bulma/css/bulma.min.css'
import './App.css'

function App() {
  const [code, setCode] = useState(`const x: number = 42;
const y: string = "hello";

function add(a: number, b: number): number {
  return a + b;
}

// This will produce an error
const result: string = add(1, 2);`);
  
  const [errors, setErrors] = useState([]);
  const [wasmModule, setWasmModule] = useState(null);
  const [isLoading, setIsLoading] = useState(true);
  const monacoRef = useRef(null);
  const editorRef = useRef(null);

  // Load WASM module on component mount
  useEffect(() => {
    const loadWasm = async () => {
      try {
        // Dynamic import of the WASM module
        const wasm = await import('./wasm/seal_cli.js');
        await wasm.default();
        setWasmModule(wasm);
        setIsLoading(false);
      } catch (error) {
        console.error('Failed to load WASM module:', error);
        setIsLoading(false);
      }
    };
    
    loadWasm();
  }, []);

  // Type check code whenever it changes
  useEffect(() => {
    if (!wasmModule || !code) return;

    const checkTypes = async () => {
      try {
        const result = wasmModule.type_check(code);
        setErrors(result.errors || []);
        updateEditorMarkers(result.errors || []);
      } catch (error) {
        console.error('Type checking failed:', error);
        setErrors([{
          message: 'Internal error: ' + error.message,
          start_line: 1,
          start_column: 1,
          end_line: 1,
          end_column: 1
        }]);
      }
    };

    // Debounce type checking
    const timer = setTimeout(checkTypes, 300);
    return () => clearTimeout(timer);
  }, [code, wasmModule]);

  const handleEditorDidMount = (editor, monaco) => {
    editorRef.current = editor;
    monacoRef.current = monaco;
  };

  const updateEditorMarkers = (errors) => {
    if (!monacoRef.current || !editorRef.current) return;

    const model = editorRef.current.getModel();
    if (!model) return;

    // Convert errors to Monaco markers
    const markers = errors.map(error => ({
      startLineNumber: error.start_line,
      startColumn: error.start_column,
      endLineNumber: error.end_line || error.start_line,
      endColumn: error.end_column || error.start_column + 1,
      message: error.message,
      severity: monacoRef.current.MarkerSeverity.Error,
    }));

    // Set markers
    monacoRef.current.editor.setModelMarkers(model, 'seal-type-checker', markers);
  };

  const handleErrorClick = (error) => {
    if (!editorRef.current) return;
    
    // Focus editor and jump to error position
    editorRef.current.focus();
    editorRef.current.setPosition({
      lineNumber: error.start_line,
      column: error.start_column
    });
    editorRef.current.revealPositionInCenter({
      lineNumber: error.start_line,
      column: error.start_column
    });
  };

  return (
    <div className="container is-fluid" style={{ padding: '2rem', maxWidth: '1600px' }}>
      <h1 className="title is-2">Seal TypeScript</h1>
      <p className="subtitle is-5">A TypeScript written in Rust, running in your browser via WebAssembly</p>
      
      {isLoading ? (
        <div className="notification is-info">
          <p>Loading WebAssembly module...</p>
        </div>
      ) : (
        <div className="columns">
          <div className="column is-two-thirds">
            <div className="field">
              <label className="label">TypeScript Code</label>
              <div className="box" style={{ padding: 0, overflow: 'hidden' }}>
                <MonacoEditor
                  height="600px"
                  value={code}
                  onChange={setCode}
                  onMount={handleEditorDidMount}
                  theme="vs-dark"
                />
              </div>
            </div>
          </div>
          
          <div className="column is-one-third">
            <div className="field">
              <label className="label">Type Checking Results</label>
              {errors.length === 0 ? (
                <div className="notification is-success">
                  <span className="icon">
                    <i className="fas fa-check-circle"></i>
                  </span>
                  <span>No type errors found!</span>
                </div>
              ) : (
                <div>
                  <div className="notification is-danger">
                    <span className="icon">
                      <i className="fas fa-exclamation-triangle"></i>
                    </span>
                    <span>{errors.length} error{errors.length > 1 ? 's' : ''} found</span>
                  </div>
                  <div className="error-list" style={{ maxHeight: '500px', overflowY: 'auto' }}>
                    {errors.map((error, index) => (
                      <div 
                        key={index} 
                        className="message is-danger error-item" 
                        style={{ marginBottom: '0.5rem', cursor: 'pointer' }}
                        onClick={() => handleErrorClick(error)}
                      >
                        <div className="message-body" style={{ padding: '0.75rem' }}>
                          <p style={{ fontFamily: 'monospace', fontSize: '12px', marginBottom: '0.25rem' }}>
                            <strong>Line {error.start_line}:{error.start_column}</strong>
                          </p>
                          <p style={{ fontSize: '14px' }}>{error.message}</p>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
      
      <footer className="footer" style={{ marginTop: '3rem', padding: '2rem 0' }}>
        <div className="content has-text-centered">
          <p>
            <strong>Seal</strong> - A TypeScript implementation in Rust
          </p>
        </div>
      </footer>
    </div>
  )
}

export default App
