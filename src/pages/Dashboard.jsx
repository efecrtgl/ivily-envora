import { useT } from '../i18n/index.jsx';
import { PageHeader } from '../components/ui/primitives.jsx';

// Pano içeriği aşama 15'te (pano ve etkinlik merkezi) gerçek verilerle doldurulur.
export default function Dashboard() {
  const t = useT();
  return (
    <div className="page">
      <PageHeader title={t('dashboard.title')} subtitle={t('dashboard.subtitle')} />
    </div>
  );
}
