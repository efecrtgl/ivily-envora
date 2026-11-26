import { useT } from '../../i18n/index.jsx';
import { Badge } from '../ui/primitives.jsx';

const ICON = { process: 'cpu', user: 'user', machine: 'monitor' };

export default function ScopeBadge({ scope }) {
  const t = useT();
  return (
    <Badge icon={ICON[scope]} title={t(`scope.${scope}Help`)} tone={scope === 'machine' ? 'solid' : undefined}>
      {t(`scope.${scope}`)}
    </Badge>
  );
}
