import { describe, expect, it, vi, afterEach } from 'vitest';
import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { I18nProvider } from '../i18n/index.jsx';
import { SettingsProvider } from '../app/settings.jsx';
import { ToastProvider } from './ui/Toast.jsx';
import SecretValue from './env/SecretValue.jsx';
import { Modal, ConfirmDialog } from './ui/Modal.jsx';
import VarRowsEditor, { rowsFromVars } from './envset/VarRowsEditor.jsx';

function wrap(ui) {
  return render(
    <SettingsProvider>
      <I18nProvider lang="tr">
        <ToastProvider>{ui}</ToastProvider>
      </I18nProvider>
    </SettingsProvider>,
  );
}

afterEach(cleanup);

describe('SecretValue', () => {
  it('maskeli değeri göstermez; açıkça istenince arka uç işlevini çağırır', async () => {
    const reveal = vi.fn().mockResolvedValue('gizli-deger');
    wrap(<SecretValue v={{ name: 'TOKEN', scope: 'user', kind: 'string', masked: true, value: null }} reveal={reveal} />);
    expect(screen.queryByText('gizli-deger')).toBeNull();
    expect(screen.getByLabelText('Gizli değer')).toBeTruthy();
    expect(reveal).not.toHaveBeenCalled();
    await act(async () => {
      fireEvent.click(screen.getByLabelText('Göster'));
    });
    expect(reveal).toHaveBeenCalledTimes(1);
    expect(screen.getByText('gizli-deger')).toBeTruthy();
    // Tekrar tıklayınca gizlenir.
    await act(async () => {
      fireEvent.click(screen.getByLabelText('Gizle'));
    });
    expect(screen.queryByText('gizli-deger')).toBeNull();
  });

  it('reveal={null} iken göster düğmesi yoktur', () => {
    wrap(<SecretValue v={{ name: 'TOKEN', scope: 'user', kind: 'string', masked: true, value: null }} reveal={null} />);
    expect(screen.queryByLabelText('Göster')).toBeNull();
  });

  it('hassas olmayan değeri olduğu gibi gösterir', () => {
    wrap(<SecretValue v={{ name: 'A', scope: 'user', kind: 'string', masked: false, value: 'C:\\x' }} />);
    expect(screen.getByText('C:\\x')).toBeTruthy();
  });
});

describe('Modal', () => {
  it('Escape ile kapanır ve odak içeride başlar', () => {
    const onClose = vi.fn();
    wrap(
      <Modal title="Başlık" onClose={onClose}>
        <input aria-label="alan" />
      </Modal>,
    );
    expect(document.activeElement).toBe(screen.getByLabelText('alan'));
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(onClose).toHaveBeenCalled();
  });

  it('ConfirmDialog meşgulken kapatılamaz', () => {
    const onCancel = vi.fn();
    wrap(
      <ConfirmDialog title="Onay" onCancel={onCancel} onConfirm={() => {}} busy>
        gövde
      </ConfirmDialog>,
    );
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(onCancel).not.toHaveBeenCalled();
  });
});

describe('VarRowsEditor', () => {
  it('satır ekler ve maskeli değeri "değişmedi" olarak gösterir', () => {
    let rows = rowsFromVars([{ name: 'TOKEN', value: null, sensitive: true, kind: null, scope: 'user', action: 'set' }]);
    const onChange = vi.fn((next) => {
      rows = next;
    });
    wrap(<VarRowsEditor rows={rows} onChange={onChange} />);
    expect(screen.getByText('Değişmedi')).toBeTruthy();
    fireEvent.click(screen.getByText('Değişken ekle'));
    expect(onChange).toHaveBeenCalled();
    expect(rows).toHaveLength(2);
    expect(rows[1]).toMatchObject({ name: '', value: '', action: 'set', scope: 'user' });
  });
});
