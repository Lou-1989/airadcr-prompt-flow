import { cn } from '@/lib/utils';

interface WebViewContainerProps {
  className?: string;
}

export const WebViewContainer = ({ className }: WebViewContainerProps) => {
  return (
    <div className={cn("h-full w-full", className)}>
      <iframe
        src="https://airadcr.com"
        className="w-full h-full border-0"
        title="AirADCR"
        allow="clipboard-read; clipboard-write; fullscreen; display-capture"
        sandbox="allow-same-origin allow-scripts allow-forms allow-navigation allow-popups allow-modals"
      />
    </div>
  );
};