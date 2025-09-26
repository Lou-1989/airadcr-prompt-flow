import { WebViewContainer } from '@/components/WebViewContainer';
import { useTauriCommands } from '@/hooks/useTauriCommands';

const Index = () => {
  const { toggleAlwaysOnTop } = useTauriCommands();

  return (
    <div className="h-screen w-screen overflow-hidden">
      <WebViewContainer className="h-full w-full" />
    </div>
  );
};

export default Index;