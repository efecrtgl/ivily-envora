import { useMemo, useState } from 'react';
import { useT } from '../../i18n/index.jsx';
import { Modal } from '../ui/Modal.jsx';
import { Button, Callout, Checkbox, Field, Segmented } from '../ui/primitives.jsx';
import { validateName, looksSensitiveName, hasPercentRef } from '../../lib/envValidate.js';

/**
 * Değişken düzenleyici. Kipler:
 *  - create:    yeni değişken (kapsam seçilebilir)
 *  - edit:      mevcut değişkeni düzenle (kapsam sabit; hassas değer önceden doldurulmaz)
 *  - duplicate: başka kapsama/ada kopyala (değer arka uçta okunur)
 *  - rename:    aynı kapsamda yeni ad (değer arka uçta okunur)
 * Kaydet, bir değişiklik planı üretip `onPlan(plan)` çağırır; uygulama ChangeFlow ile yapılır.
 */
export default function VariableEditor({ mode, initial = {}, onPlan, onClose, canWriteMachine }) {
  const t = useT();
  const [scope, setScope] = useState(initial.scope && initial.scope !== 'process' ? initial.scope : 'user');
  const [name, setName] = useState(mode === 'duplicate' || mode === 'rename' ? '' : initial.name || '');
  const [value, setValue] = useState((mode === 'edit' && !initial.masked) || mode === 'create' ? initial.value ?? '' : '');
  const [keepValue, setKeepValue] = useState(mode === 'edit' && initial.masked);
  const [kind, setKind] = useState(mode === 'edit' ? initial.kind : 'auto');
  const [showValue, setShowValue] = useState(false);
  const [touched, setTouched] = useState(false);

  const nameError = validateName(name);
  const sensitiveInput = initial.masked || looksSensitiveName(name);
  const sameAsSource = (mode === 'duplicate' && scope === initial.scope && name.toUpperCase() === (initial.name || '').toUpperCase()) ||
    (mode === 'rename' && name.toUpperCase() === (initial.name || '').toUpperCase());
  const valueHasNul = value.includes('\0');
  const valid = !nameError && !sameAsSource && !valueHasNul;

  const plan = useMemo(() => {
    const k = kind === 'auto' ? null : kind;
    switch (mode) {
      case 'duplicate':
        return { scope, ops: [{ op: 'copy', name: name.trim(), sourceScope: initial.scope, sourceName: initial.name }] };
      case 'rename':
        return {
          scope: initial.scope,
          ops: [
            { op: 'copy', name: name.trim(), sourceScope: initial.scope, sourceName: initial.name },
            { op: 'delete', name: initial.name },
          ],
        };
      case 'edit':
        return { scope: initial.scope, ops: [{ op: 'set', name: initial.name, value: keepValue ? null : value, kind: k }] };
      default:
        return { scope, ops: [{ op: 'set', name: name.trim(), value, kind: k }] };
    }
  }, [mode, scope, name, value, keepValue, kind, initial]);

  const submit = (e) => {
    e?.preventDefault();
    setTouched(true);
    if (valid) onPlan(plan);
  };

  const titleKey = { create: 'editor.createTitle', edit: 'editor.editTitle', duplicate: 'editor.duplicateTitle', rename: 'editor.renameTitle' }[mode];
  const scopeOptions = [
    { value: 'user', label: t('scope.user') },
    { value: 'machine', label: t('scope.machine') },
  ];
  const showValueField = mode === 'create' || mode === 'edit';
  const showKind = mode === 'create' || mode === 'edit';

  return (
    <Modal
      title={t(titleKey, { name: initial.name })}
      onClose={onClose}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t('common.cancel')}
          </Button>
          <Button variant="primary" onClick={submit} disabled={touched && !valid}>
            {t('editor.review')}
          </Button>
        </>
      }
    >
      <form className="stack" onSubmit={submit}>
        {(mode === 'create' || mode === 'duplicate') && (
          <Field label={t('editor.targetScope')} hint={scope === 'machine' && !canWriteMachine ? t('editor.machineHint') : undefined}>
            <Segmented label={t('editor.targetScope')} value={scope} onChange={setScope} options={scopeOptions} />
          </Field>
        )}
        {mode !== 'edit' && (
          <Field label={mode === 'create' ? t('common.name') : t('editor.newName')} error={touched && (nameError ? t(`errors.${nameError}`) : sameAsSource ? t('editor.sameName') : null)} htmlFor="var-name">
            <input
              id="var-name"
              className="input mono"
              value={name}
              onChange={(e) => setName(e.target.value)}
              spellCheck={false}
              autoComplete="off"
              placeholder="JAVA_HOME"
              aria-invalid={touched && !!(nameError || sameAsSource)}
            />
          </Field>
        )}
        {(mode === 'duplicate' || mode === 'rename') && <Callout>{t('editor.copyNote')}</Callout>}
        {showValueField && (
          <Field label={t('common.value')} error={valueHasNul ? t('errors.invalid_value_nul') : null} hint={sensitiveInput ? t('editor.sensitiveHint') : undefined}>
            {mode === 'edit' && initial.masked && (
              <Checkbox checked={keepValue} onChange={setKeepValue}>
                {t('editor.keepValue')}
              </Checkbox>
            )}
            {!keepValue &&
              (sensitiveInput && !showValue ? (
                <div className="row" style={{ flexWrap: 'nowrap' }}>
                  <input className="input mono" type="password" value={value} onChange={(e) => setValue(e.target.value)} autoComplete="new-password" />
                  <Button size="sm" variant="ghost" iconOnly icon="eye" title={t('common.reveal')} onClick={() => setShowValue(true)} />
                </div>
              ) : (
                <textarea className="textarea mono" rows={3} value={value} onChange={(e) => setValue(e.target.value)} spellCheck={false} />
              ))}
          </Field>
        )}
        {showKind && (
          <Field label={t('editor.kind')} hint={kind === 'auto' ? t('editor.kindAutoHint') : t('kind.expandHelp')}>
            <Segmented
              label={t('editor.kind')}
              value={kind}
              onChange={setKind}
              options={[
                ...(mode === 'create' ? [{ value: 'auto', label: t('editor.kindAuto') }] : []),
                { value: 'string', label: t('kind.string') },
                { value: 'expand', label: t('kind.expand') },
              ]}
            />
            {kind === 'string' && hasPercentRef(value) && <span className="field-hint" style={{ color: 'var(--warn)' }}>{t('issue.unexpandedReference')}</span>}
          </Field>
        )}
        <button type="submit" hidden />
      </form>
    </Modal>
  );
}
