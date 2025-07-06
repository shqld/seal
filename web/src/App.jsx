import { useState, useEffect } from 'react'
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

  const getLineNumbers = () => {
    const lines = code.split('\n');
    return lines.map((_, index) => index + 1).join('\n');
  };

  return (
    <div className="container is-fluid" style={{ padding: '2rem' }}>
      <h1 className="title is-1">Seal TypeScript Type Checker</h1>
      <p className="subtitle">A TypeScript type checker written in Rust, running in your browser via WebAssembly</p>
      
      {isLoading ? (
        <div className="notification is-info">
          <p>Loading WebAssembly module...</p>
        </div>
      ) : (
        <div className="columns">
          <div className="column is-two-thirds">
            <div className="field">
              <label className="label">TypeScript Code</label>
              <div className="control">
                <div style={{ display: 'flex', fontFamily: 'monospace', fontSize: '14px' }}>
                  <pre 
                    style={{ 
                      margin: 0, 
                      padding: '10px', 
                      backgroundColor: '#f5f5f5',
                      color: '#666',
                      borderRight: '1px solid #ddd',
                      minWidth: '40px',
                      textAlign: 'right'
                    }}
                  >
                    {getLineNumbers()}
                  </pre>
                  <textarea
                    className="textarea"
                    value={code}
                    onChange={(e) => setCode(e.target.value)}
                    rows={20}
                    style={{ 
                      fontFamily: 'monospace',
                      fontSize: '14px',
                      borderRadius: '0 4px 4px 0',
                      resize: 'vertical'
                    }}
                    spellCheck={false}
                  />
                </div>
              </div>
            </div>
          </div>
          
          <div className="column is-one-third">
            <div className="field">
              <label className="label">Type Checking Results</label>
              {errors.length === 0 ? (
                <div className="notification is-success">
                  ✓ No type errors found!
                </div>
              ) : (
                <div>
                  <div className="notification is-danger">
                    {errors.length} error{errors.length > 1 ? 's' : ''} found
                  </div>
                  {errors.map((error, index) => (
                    <div key={index} className="message is-danger" style={{ marginBottom: '1rem' }}>
                      <div className="message-body">
                        <p style={{ fontFamily: 'monospace', fontSize: '13px' }}>
                          <strong>Line {error.start_line}:{error.start_column}</strong>
                        </p>
                        <p>{error.message}</p>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      )}
      
      <footer className="footer" style={{ marginTop: '3rem', padding: '2rem 0' }}>
        <div className="content has-text-centered">
          <p>
            <strong>Seal</strong> - A TypeScript type checker implementation in Rust
          </p>
        </div>
      </footer>
    </div>
  )
}

export default App