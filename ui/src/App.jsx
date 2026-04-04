import React, { useState } from 'react';
import { Terminal, Database } from 'lucide-react';
import RepositoryIngester from './components/RepositoryIngester';
import DiagnosticConsole from './components/DiagnosticConsole';
import ModelSettings from './components/ModelSettings';
import RepoContext from './components/RepoContext';

function App() {
  const [isIngested, setIsIngested] = useState(false);
  const [activeRepo, setActiveRepo] = useState(null);
  const [aiConfig, setAiConfig] = useState({
    url: 'http://localhost:11434/v1/chat/completions',
    model: 'llama3.1:8b',
    key: 'ollama'
  });

  const handleIngestSuccess = (repoUrl) => {
    setIsIngested(true);
    setActiveRepo(repoUrl);
  };

  return (
    <div className="app-container">
      {/* Sidebar: Ingestion Settings */}
      <div className="glass-panel" style={{ overflowY: 'auto' }}>
        <h2 className="section-title">
          <Database size={20} className="text-cyan" />
          CodePath Ingestion
        </h2>
        
        <RepositoryIngester 
          onSuccess={handleIngestSuccess} 
          isIngested={isIngested}
          activeRepo={activeRepo}
        />

        <ModelSettings aiConfig={aiConfig} setAiConfig={setAiConfig} />

        <RepoContext activeRepo={activeRepo} />
      </div>

      {/* Main Container: Diagnostics */}
      <div className="glass-panel" style={{ padding: '0' }}>
        <h2 className="section-title" style={{ margin: '1.5rem', marginBottom: 0 }}>
          <Terminal size={20} className="text-cyan" />
          Intelligence Evaluator Console
        </h2>
        
        <DiagnosticConsole isEngineReady={isIngested} aiConfig={aiConfig} />
      </div>
    </div>
  );
}

export default App;
