import Editor, { useMonaco } from '@monaco-editor/react'
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker'
import jsonWorker from 'monaco-editor/esm/vs/language/json/json.worker?worker';
import tsWorker from 'monaco-editor/esm/vs/language/typescript/ts.worker?worker';
import { useEffect } from 'react';

self.MonacoEnvironment = {
  getWorker(_, label) {
    switch (label) {
      case 'json':
        return new jsonWorker();
      case "typescript":
      case "javascript":
        return new tsWorker();
      case "editorWorkerService":
        return new editorWorker();
      default:
        throw new Error(`Unknown worker label: ${label}`);
    }
  }
}


function MonacoEditor({ value, onChange, onMount, theme = 'vs-dark', ...props }) {
  const monaco = useMonaco();

  useEffect(() => {
    monaco?.languages.typescript.typescriptDefaults.setModeConfiguration({ 
        completionItems: true,
        hovers: true,
        documentSymbols: false,
        definitions: false,
        references: false,
        documentHighlights: false,
        rename: false,
        diagnostics: false,
        documentRangeFormattingEdits: false,
        signatureHelp: false,
        onTypeFormattingEdits: false,
        codeActions: false,
        inlayHints: false,
    })
  }, [monaco]);


  return (
    <Editor
      language="typescript"
      value={value}
      onChange={onChange}
      onMount={onMount}
      theme={theme}
      options={{
        minimap: { enabled: false },
        fontSize: 14,
        lineNumbers: 'on',
        scrollBeyondLastLine: false,
        automaticLayout: true,
        tabSize: 2,
        wordWrap: 'on',
        'semanticHighlighting.enabled': true,
        quickSuggestions: {
          other: true,
          comments: false,
          strings: false
        },
        parameterHints: {
          enabled: true
        },
        suggestOnTriggerCharacters: true,
        acceptSuggestionOnCommitCharacter: true,
        acceptSuggestionOnEnter: 'on',
        accessibilitySupport: 'auto',
        ...props.options
      }}
      {...props}
    />
  )
}

export default MonacoEditor
