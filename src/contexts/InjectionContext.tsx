import { createContext, useContext, ReactNode } from 'react';
import { useInjection } from '@/hooks/useInjection';

interface InjectionContextType {
  getCursorPosition: () => Promise<{ x: number; y: number } | null>;
  performInjection: (text: string) => Promise<boolean>;
  testInjectionAvailability: () => Promise<boolean>;
  externalPositions: Array<{ x: number; y: number; timestamp: number }>;
  isMonitoring: boolean;
  startMonitoring: () => void;
  stopMonitoring: () => void;
  lockCurrentPosition: () => Promise<boolean>;
  unlockPosition: () => void;
  updateLockedPosition: () => Promise<boolean>;
  isLocked: boolean;
  lockedPosition: any;
  isInjecting: boolean;
}

const InjectionContext = createContext<InjectionContextType | null>(null);

export const InjectionProvider = ({ children }: { children: ReactNode }) => {
  const injection = useInjection();
  
  return (
    <InjectionContext.Provider value={injection}>
      {children}
    </InjectionContext.Provider>
  );
};

export const useInjectionContext = () => {
  const context = useContext(InjectionContext);
  if (!context) {
    throw new Error('useInjectionContext doit être utilisé dans InjectionProvider');
  }
  return context;
};
