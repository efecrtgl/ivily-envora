import { useState } from 'react';
import { useT } from '../../i18n/index.jsx';
import { Button, Card, PageHeader } from '../../components/ui/primitives.jsx';
import ExportDialog from '../../components/transfer/ExportDialog.jsx';
import ImportPanel from './ImportPanel.jsx';

export default function Transfer() {
  const t = useT();
  const [exporting, setExporting] = useState(null);
  return (
    <div className="page">
      <PageHeader title={t('transfer.title')} subtitle={t('transfer.subtitle')} />
      <div className="stack">
        <Card title={t('transfer.exportCard')} icon="download" desc={t('transfer.exportDesc')}>
          <div className="row">
            <Button icon="variable" onClick={() => setExporting({ kind: 'variables' })}>
              {t('transfer.exportVariables')}
            </Button>
            <Button icon="path" onClick={() => setExporting({ kind: 'path', scope: 'user' })}>
              {t('transfer.exportPathUser')}
            </Button>
            <Button icon="path" onClick={() => setExporting({ kind: 'path', scope: 'machine' })}>
              {t('transfer.exportPathMachine')}
            </Button>
          </div>
          <p className="xsmall faint" style={{ marginTop: 10 }}>
            {t('transfer.exportOthers')}
          </p>
        </Card>
        <ImportPanel />
      </div>
      {exporting && <ExportDialog preset={exporting} onClose={() => setExporting(null)} />}
    </div>
  );
}
