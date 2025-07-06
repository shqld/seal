import Editor from '@monaco-editor/react'

// Configure Monaco Editor Workers
self.MonacoEnvironment = {
  getWorkerUrl() {
    return './editor.worker.bundle.js'
  }
}

function MonacoEditor({ value, onChange, onMount, theme = 'vs-dark', ...props }) {
  return (
    <Editor
      defaultLanguage="typescript"
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
