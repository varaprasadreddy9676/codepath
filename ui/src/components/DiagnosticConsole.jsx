import React, { useState, useRef, useEffect } from 'react';
import { Send, Zap } from 'lucide-react';
import ReactMarkdown from 'react-markdown';

const API_BASE = 'http://127.0.0.1:3000/api/v1';

export default function DiagnosticConsole({ isEngineReady, aiConfig }) {
  const [query, setQuery] = useState('');
  const [messages, setMessages] = useState([
    { role: 'ai', content: '**CodePath Generic Intelligence Engine v1.0** \n\nPlease provide a repository absolute path in the left panel to initialize the generic AST parsers. Once successfully indexed, paste your stacktraces or arbitrary logic questions directly here.' }
  ]);
  const [isInferring, setIsInferring] = useState(false);
  const endRef = useRef(null);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, isInferring]);

  const handleSubmit = async (e) => {
    e.preventDefault();
    if (!query.trim() || !isEngineReady) return;

    const userPayload = query.trim();
    setMessages(prev => [...prev, { role: 'user', content: userPayload }]);
    setQuery('');
    setIsInferring(true);

    try {
      const resp = await fetch(`${API_BASE}/investigate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
            text: userPayload,
            llm_api_url: aiConfig.url,
            llm_model: aiConfig.model,
            llm_api_key: aiConfig.key
        })
      });
      
      const data = await resp.json();
      
      if (resp.ok && data.result) {
        setMessages(prev => [...prev, { role: 'ai', content: data.result }]);
      } else {
        setMessages(prev => [...prev, { role: 'ai', content: `**Axum Structural Exception:** The backend evaluated failure: ${resp.statusText}` }]);
      }
    } catch (err) {
      setMessages(prev => [...prev, { role: 'ai', content: `**Network Unreachable Exception:** Axum server socket bridge is definitively offline at port 3000. Navigate to terminal and verify \`cargo run\` executed.` }]);
    } finally {
      setIsInferring(false);
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', padding: '1.5rem', paddingTop: '1rem' }}>
      
      {!isEngineReady && (
        <div style={{ background: 'rgba(255, 100, 100, 0.1)', border: '1px solid rgba(255,100,100,0.3)', color: '#ff8a8a', padding: '0.85rem', borderRadius: '8px', fontSize: '0.9rem', marginBottom: '1rem', display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
          <Zap size={18} /> Engine Evaluator Pipeline locked until a valid repository infrastructure is verified.
        </div>
      )}

      <div className="chat-window">
        {messages.map((msg, i) => (
          <div key={i} className={`message ${msg.role}`}>
            {msg.role === 'ai' ? (
               <ReactMarkdown>{msg.content}</ReactMarkdown>
            ) : (
               msg.content
            )}
          </div>
        ))}
        {isInferring && (
          <div className="message ai typing-indicator">
            <div className="dot"></div>
            <div className="dot"></div>
            <div className="dot"></div>
          </div>
        )}
        <div ref={endRef} />
      </div>

      <form onSubmit={handleSubmit} style={{ display: 'flex', gap: '0.75rem', marginTop: 'auto' }}>
        <input 
          type="text"
          className="glass-input"
          placeholder={isEngineReady ? "Paste an unconstrained stacktrace or ask 'Why is this status DRAFT?'..." : "Awaiting repository AST mappings..."}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          disabled={!isEngineReady || isInferring}
        />
        <button 
          type="submit" 
          className="cyber-button" 
          style={{ width: 'auto', marginTop: 0, padding: '0 1.25rem' }}
          disabled={!isEngineReady || isInferring || !query.trim()}
        >
          <Send size={18} />
        </button>
      </form>
    </div>
  );
}
