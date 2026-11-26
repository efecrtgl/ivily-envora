import { useEffect, useState } from 'react';
import { useT, LANGUAGES } from '../i18n/index.jsx';
import { useSettings } from '../app/settings.jsx';
import { api } from '../lib/api.js';
import { Card, Field, PageHeader, Segmented } from '../components/ui/primitives.jsx';

export default function Settings() {
  const t = useT();
  const { ui, update } = useSettings();
  const [info, setInfo] = useState(null);

  useEffect(() => {
    api.appInfo().then(setInfo).catch(() => setInfo(null));
  }, []);

  return (
    <div className="page">
      <PageHeader title={t('settings.title')} subtitle={t('settings.subtitle')} />
      <div className="stack">
        <Card title={t('settings.appearance')} icon="sun">
          <div className="grid cols-2">
            <Field label={t('theme.label')} hint={t('settings.themeHint')}>
              <Segmented
                label={t('theme.label')}
                value={ui.theme}
                onChange={(theme) => update({ theme })}
                options={['system', 'dark', 'light'].map((v) => ({ value: v, label: t(`theme.${v}`) }))}
              />
            </Field>
            <Field label={t('settings.language')} hint={t('settings.languageHint')}>
              <Segmented
                label={t('settings.language')}
                value={ui.language}
                onChange={(language) => update({ language })}
                options={LANGUAGES}
              />
            </Field>
          </div>
        </Card>
        <Card title={t('settings.about')} icon="info">
          <dl className="kv">
            <dt>{t('common.version')}</dt>
            <dd>{info?.version || __APP_VERSION__}</dd>
            <dt>{t('settings.platform')}</dt>
            <dd>{info ? `${info.platform} / ${info.arch}` : '—'}</dd>
          </dl>
        </Card>
      </div>
    </div>
  );
}
