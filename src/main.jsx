import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import App from './app/App.jsx';
import './styles/tokens.css';
import './styles/base.css';
import './styles/components.css';

// Sağ tık menüsünü yalnızca metin alanlarında bırak (masaüstü uygulaması hissi).
document.addEventListener('contextmenu', (e) => {
  if (!e.target.closest('input, textarea, .selectable')) e.preventDefault();
});

createRoot(document.getElementById('root')).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
