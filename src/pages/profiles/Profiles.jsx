import { useNav } from '../../app/navigation.js';
import ProfileList from './ProfileList.jsx';
import ProfileDetail from './ProfileDetail.jsx';

/** Profil listesi ve ayrıntısı arasında `params.id` ile geçiş yapar. */
export default function Profiles({ params }) {
  const { navigate } = useNav();
  const open = (id) => navigate('profiles', id ? { id } : null);
  if (params?.id) return <ProfileDetail key={params.id} id={params.id} onBack={open} />;
  return <ProfileList onOpen={open} />;
}
