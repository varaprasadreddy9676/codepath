import React, { useState } from 'react';
import { Package, Loader2, Copy, CheckCircle2, FileCode, GitBranch, Shrink, Hash } from 'lucide-react';

const API_BASE = 'http://127.0.0.1:3000/api/v1';

export default function RepoContext({ activeRepo }) {
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState(null);
  const [copied, setCopied] = useState(false);
  const [options, setOptions] = useState({
    style: 'xml',
    compress: false,
    include_git_log: true,
    include_git_diff: false,
    include_repo_map: true,
    include_tree: true,
    show_line_numbers: false,
    include_patterns: '',
    exclude_patterns: '',
  });

  const handlePack = async () => {
    if (!activeRepo) return;
    setLoading(true);
    setResult(null);

    try {
      const body = {
        repo_path: activeRepo,
        style: options.style,
        compress: options.compress,
        include_git_log: options.include_git_log,
        include_git_diff: options.include_git_diff,
        include_repo_map: options.include_repo_map,
        include_tree: options.include_tree,
        show_line_numbers: options.show_line_numbers,
      };

      if (options.include_patterns.trim()) {
        body.include_patterns = options.include_patterns.split(',').map(s => s.trim()).filter(Boolean);
      }
      if (options.exclude_patterns.trim()) {
        body.exclude_patterns = options.exclude_patterns.split(',').map(s => s.trim()).filter(Boolean);
      }

      const resp = await fetch(`${API_BASE}/pack`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      });

      const data = await resp.json();
      if (resp.ok) {
        setResult(data);
      } else {
        setResult({ error: 'Pack failed: ' + (data.message || resp.statusText) });
      }
    } catch (err) {
      setResult({ error: `Connection failed: ${err.message}` });
    } finally {
      setLoading(false);
    }
  };

  const handleCopy = () => {
    if (result?.content) {
      navigator.clipboard.writeText(result.content);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const formatTokens = (n) => {
    if (n >= 1000000) return (n / 1000000).toFixed(1) + 'M';
    if (n >= 1000) return (n / 1000).toFixed(1) + 'k';
    return n;
  };

  return (
    <div style={{ marginTop: '1.5rem' }}>
      <h2 className="section-title" style={{ fontSize: '0.95rem' }}>
        <Package size={18} className="text-cyan" />
        Context Pack Engine
      </h2>

      {/* Output Style */}
      <div className="input-group" style={{ marginBottom: '0.5rem' }}>
        <label className="input-label">Output Format</label>
        <div style={{ display: 'flex', gap: '0.5rem' }}>
          {['xml', 'markdown', 'plain'].map(s => (
            <button
              key={s}
              type="button"
              onClick={() => setOptions({ ...options, style: s })}
              style={{
                flex: 1, padding: '0.4rem', borderRadius: '6px', border: '1px solid',
                borderColor: options.style === s ? 'var(--accent-cyan)' : 'var(--border-light)',
                background: options.style === s ? 'rgba(0,240,255,0.1)' : 'rgba(0,0,0,0.3)',
                color: options.style === s ? 'var(--accent-cyan)' : 'var(--text-muted)',
                cursor: 'pointer', fontSize: '0.8rem', fontWeight: 600, textTransform: 'uppercase',
              }}
            >
              {s}
            </button>
          ))}
        </div>
      </div>

      {/* Toggle Options */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '0.35rem', marginBottom: '0.5rem' }}>
        {[
          { key: 'compress', label: 'Compress', icon: <Shrink size={12} /> },
          { key: 'include_repo_map', label: 'Repo Map', icon: <FileCode size={12} /> },
          { key: 'include_tree', label: 'Dir Tree', icon: <Hash size={12} /> },
          { key: 'include_git_log', label: 'Git Log', icon: <GitBranch size={12} /> },
          { key: 'include_git_diff', label: 'Git Diff', icon: <GitBranch size={12} /> },
          { key: 'show_line_numbers', label: 'Line #s', icon: <Hash size={12} /> },
        ].map(({ key, label, icon }) => (
          <label key={key} style={{
            display: 'flex', alignItems: 'center', gap: '0.35rem',
            fontSize: '0.78rem', color: options[key] ? 'var(--accent-cyan)' : 'var(--text-muted)',
            cursor: 'pointer', padding: '0.25rem 0',
          }}>
            <input
              type="checkbox"
              checked={options[key]}
              onChange={() => setOptions({ ...options, [key]: !options[key] })}
              style={{ accentColor: 'var(--accent-cyan)', width: '14px', height: '14px' }}
            />
            {icon} {label}
          </label>
        ))}
      </div>

      {/* Glob Filters */}
      <div className="input-group" style={{ marginBottom: '0.4rem' }}>
        <label className="input-label" style={{ fontSize: '0.75rem' }}>Include Globs</label>
        <input
          type="text"
          className="glass-input"
          style={{ padding: '0.5rem 0.75rem', fontSize: '0.8rem' }}
          value={options.include_patterns}
          onChange={(e) => setOptions({ ...options, include_patterns: e.target.value })}
          placeholder="e.g. **/*.rs, src/**"
        />
      </div>
      <div className="input-group" style={{ marginBottom: '0.5rem' }}>
        <label className="input-label" style={{ fontSize: '0.75rem' }}>Exclude Globs</label>
        <input
          type="text"
          className="glass-input"
          style={{ padding: '0.5rem 0.75rem', fontSize: '0.8rem' }}
          value={options.exclude_patterns}
          onChange={(e) => setOptions({ ...options, exclude_patterns: e.target.value })}
          placeholder="e.g. **/test/**, **/*.lock"
        />
      </div>

      {/* Pack Button */}
      <button
        type="button"
        className="cyber-button"
        disabled={!activeRepo || loading}
        onClick={handlePack}
        style={{ marginTop: '0.5rem' }}
      >
        {loading ? <Loader2 size={16} className="lucide-spin" /> : <Package size={16} />}
        {loading ? 'Packing...' : 'Pack Repository Context'}
      </button>

      {/* Results */}
      {result && !result.error && (
        <div style={{ marginTop: '0.75rem' }}>
          <div style={{
            display: 'flex', justifyContent: 'space-between', alignItems: 'center',
            background: 'rgba(0,240,255,0.05)', border: '1px solid rgba(0,240,255,0.15)',
            borderRadius: '8px', padding: '0.6rem 0.8rem', marginBottom: '0.5rem',
          }}>
            <div style={{ fontSize: '0.8rem', color: 'var(--accent-cyan)' }}>
              <CheckCircle2 size={14} style={{ display: 'inline', marginRight: '4px', verticalAlign: 'middle' }} />
              <strong>{result.file_count}</strong> files · <strong>{formatTokens(result.total_tokens)}</strong> tokens · {result.style}
            </div>
            <button
              onClick={handleCopy}
              style={{
                background: 'none', border: '1px solid var(--border-light)',
                color: copied ? 'var(--accent-cyan)' : 'var(--text-muted)',
                padding: '0.3rem 0.6rem', borderRadius: '6px', cursor: 'pointer',
                fontSize: '0.75rem', display: 'flex', alignItems: 'center', gap: '4px'
              }}
            >
              {copied ? <CheckCircle2 size={12} /> : <Copy size={12} />}
              {copied ? 'Copied!' : 'Copy'}
            </button>
          </div>
          <div style={{
            background: 'rgba(0,0,0,0.4)', border: '1px solid var(--border-light)',
            borderRadius: '8px', padding: '0.75rem', maxHeight: '200px', overflowY: 'auto',
            fontSize: '0.72rem', fontFamily: 'monospace', color: 'var(--text-muted)',
            whiteSpace: 'pre-wrap', lineHeight: 1.4,
          }}>
            {result.content.substring(0, 3000)}
            {result.content.length > 3000 && '\n\n... (truncated preview — full content copied to clipboard)'}
          </div>
        </div>
      )}

      {result?.error && (
        <div style={{
          marginTop: '0.75rem', fontSize: '0.82rem', color: '#ff8a8a',
          background: 'rgba(255,100,100,0.08)', padding: '0.6rem', borderRadius: '8px',
        }}>
          {result.error}
        </div>
      )}
    </div>
  );
}
