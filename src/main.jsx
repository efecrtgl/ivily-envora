import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import App from './app/App.jsx';
import { loadLanguage } from './i18n/index.jsx';
import './styles/tokens.css';
import './styles/base.css';
import './styles/components.css';

// Sağ tık menüsünü yalnızca metin alanlarında bırak (masaüstü uygulaması hissi).
document.addEventListener('contextmenu', (e) => {
  if (!e.target.closest('input, textarea, .selectable')) e.preventDefault();
});

// Kayıtlı dil Türkçe değilse sözlüğünü ilk çizimden önce yükle (dil değişimi titremesin).
let savedLang = 'tr';
try {
  savedLang = JSON.parse(localStorage.getItem('envora.ui') || '{}').language || 'tr';
} catch {
  /* yerel depolama yoksa varsayılan dil */
}

loadLanguage(savedLang)
  .catch(() => {})
  .finally(() => {
    createRoot(document.getElementById('root')).render(
      <StrictMode>
        <App />
      </StrictMode>,
    );
  });
