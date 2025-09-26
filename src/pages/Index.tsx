import { WebViewContainer } from '@/components/WebViewContainer';
import { TauriControls } from '@/components/TauriControls';

const Index = () => {
  return (
    <div className="h-screen w-screen overflow-hidden relative">
      <WebViewContainer className="h-full w-full" />
      <TauriControls />
    </div>
  );
};

export default Index;