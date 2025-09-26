import { WebViewContainer } from '@/components/WebViewContainer';
import { TauriControls } from '@/components/TauriControls';
import { PositionIndicator } from '@/components/PositionIndicator';

const Index = () => {
  return (
    <div className="h-screen w-screen overflow-hidden relative">
      <WebViewContainer className="h-full w-full" />
      <TauriControls />
      <PositionIndicator />
    </div>
  );
};

export default Index;