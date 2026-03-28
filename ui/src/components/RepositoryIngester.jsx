import React, { useState } from 'react';
import { FolderGit2, Loader2, CheckCircle2 } from 'lucide-react';

const API_BASE = 'http://127.0.0.1:3000/api/v1';

export default function RepositoryIngester({ onSuccess, isIngested, activeRepo }) {
  const [path, setPath] = useState('/Users/sai/dev/work/data-exporter');
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState('');

  const handleIngest = async (e) => {
    e.preventDefault();
    if (!path.trim()) return;

    setLoading(true);
    setStatus('Dispatching AST parsing crawlers into architecture...');

    try {
      const resp = await fetch(`${API_BASE}/ingest`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ repo_url: path, branch: 'local' })
      });
      
      const data = await resp.json();
      
      if (resp.ok) {
        setStatus(`Job ID: ${data.job_id} active. Vectors extracting in background.`);
        setTimeout(() => {
          onSuccess(path);
          setLoading(false);
        }, 1500); // Simulate network latency resolving for UX
      } else {
        setStatus('Remote backend connection routing failed.');
        setLoading(false);
      }
    } catch (err) {
      setStatus(`CORS Exception: Ensure Axum Local API is running at 127.0.0.1:3000. ${err.message}`);
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleIngest} style={{ marginTop: '1rem', flex: 1, display: 'flex', flexDirection: 'column' }}>
      <div className="input-group">
        <label className="input-label">Absolute Project Pathway</label>
        <input 
          type="text" 
          className="glass-input" 
          value={path}
          onChange={(e) => setPath(e.target.value)}
          placeholder="/Users/sai/repo/"
          disabled={isIngested || loading}
        />
      </div>
      
      {status && (
        <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)', marginBottom: '1rem', lineHeight: 1.5 }}>
          {status}
        </div>
      )}

      <div style={{ marginTop: 'auto' }}>
        {isIngested ? (
          <div className="glass-panel" style={{ background: 'rgba(0, 240, 255, 0.05)', padding: '1rem', display: 'flex', alignItems: 'center', gap: '0.5rem', color: 'var(--accent-cyan)' }}>
            <CheckCircle2 size={18} />
            <span style={{ fontSize: '0.9rem', fontWeight: 600 }}>AST Map Encoded: {activeRepo.split('/').pop()}</span>
          </div>
        ) : (
          <button type="submit" className="cyber-button" disabled={loading || !path.trim()}>
            {loading ? <Loader2 size={18} className="lucide-spin" /> : <FolderGit2 size={18} />}
            {loading ? 'Executing Indexers...' : 'Execute Local Crawl'}
          </button>
        )}
      </div>
    </form>
  );
}
