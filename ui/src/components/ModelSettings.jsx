import React from 'react';
import { Settings2 } from 'lucide-react';

export default function ModelSettings({ aiConfig, setAiConfig }) {
  return (
    <div style={{ marginTop: '2rem' }}>
      <h2 className="section-title">
        <Settings2 size={20} className="text-cyan" />
        AI Engine Configuration
      </h2>
      <form style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
        <div className="input-group" style={{ marginBottom: 0 }}>
          <label className="input-label">Provider API Server</label>
          <input 
            type="text" 
            className="glass-input" 
            value={aiConfig.url}
            onChange={(e) => setAiConfig({...aiConfig, url: e.target.value})}
            placeholder="http://localhost:11434/v1/chat/completions"
          />
        </div>
        <div className="input-group" style={{ marginBottom: 0 }}>
          <label className="input-label">Target Model Tag</label>
          <input 
            type="text" 
            className="glass-input" 
            value={aiConfig.model}
            onChange={(e) => setAiConfig({...aiConfig, model: e.target.value})}
            placeholder="llama3.1:8b"
          />
        </div>
        <div className="input-group" style={{ marginBottom: 0 }}>
          <label className="input-label">Bearer Token</label>
          <input 
            type="password" 
            className="glass-input" 
            value={aiConfig.key}
            onChange={(e) => setAiConfig({...aiConfig, key: e.target.value})}
            placeholder="ollama (or sk-...)"
          />
        </div>
      </form>
    </div>
  );
}
