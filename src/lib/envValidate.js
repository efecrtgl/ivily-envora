// Ön uçta anında geri bildirim için ad doğrulaması. Asıl doğrulama arka uçta yapılır;
// bu kurallar `src-tauri/src/env/validate.rs` ile uyumlu tutulmalıdır.

export const MAX_NAME_LEN = 255;

/** Hata kodu (errors.<kod>) veya null döndürür. */
export function validateName(name) {
  if (!name || !name.trim()) return 'invalid_name_empty';
  if ([...name].length > MAX_NAME_LEN) return 'invalid_name_length';
  if (name.includes('=')) return 'invalid_name_equals';
  // eslint-disable-next-line no-control-regex
  if (/[\u0000-\u001f\u007f]/.test(name)) return 'invalid_name_control';
  return null;
}

const SENSITIVE_WORDS = ['SECRET', 'PASSWORD', 'PASSWD', 'PASS', 'TOKEN', 'APIKEY', 'CREDENTIAL', 'CREDENTIALS', 'PASSPHRASE', 'AUTH', 'DSN'];

/** Giriş alanının parola biçiminde gösterilip gösterilmeyeceğine karar vermek için kaba tahmin. */
export function looksSensitiveName(name) {
  const parts = (name || '').toUpperCase().split(/[^A-Z0-9]+/).filter(Boolean);
  if (parts.some((p) => SENSITIVE_WORDS.includes(p))) return true;
  const joined = parts.join('_');
  return /(API_KEY|ACCESS_KEY|PRIVATE_KEY|SECRET_KEY|CLIENT_SECRET|DATABASE_URL)/.test(joined);
}

export function hasPercentRef(value) {
  return /%[^%\s]+%/.test(value || '');
}
