import { createContext, useContext } from 'react';

export const NavContext = createContext({ page: 'dashboard', params: null, navigate: () => {} });

export function useNav() {
  return useContext(NavContext);
}
