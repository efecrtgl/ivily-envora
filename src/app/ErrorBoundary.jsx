import { Component } from 'react';
import { translate } from '../i18n/index.jsx';

// Bir sayfadaki beklenmeyen hata tüm uygulamayı çökertmesin.
// Hata ayrıntısı ekrana yazılmaz (değişken değeri içerebilir); yalnızca genel bir mesaj gösterilir.
export default class ErrorBoundary extends Component {
  state = { failed: false };

  static getDerivedStateFromError() {
    return { failed: true };
  }

  render() {
    if (!this.state.failed) return this.props.children;
    const lang = document.documentElement.lang || 'tr';
    return (
      <div className="page">
        <div className="card">
          <div className="empty">
            <h3>{translate(lang, 'errors.internal')}</h3>
            <button className="btn" onClick={() => this.setState({ failed: false })}>
              {translate(lang, 'common.retry')}
            </button>
          </div>
        </div>
      </div>
    );
  }
}
