import { cn } from '@/lib/utils';
import { PRODUCTION_CONFIG } from '@/config/production';

interface WebViewContainerProps {
  className?: string;
}

export const WebViewContainer = ({ className }: WebViewContainerProps) => {
  return (
    <div className={cn("h-full w-full", className)}>
      <iframe
        src={PRODUCTION_CONFIG.AIRADCR_URL}
        className="w-full h-full border-0"
        title="AirADCR"
        allow={PRODUCTION_CONFIG.IFRAME_CONFIG.allow}
        sandbox={PRODUCTION_CONFIG.IFRAME_CONFIG.sandbox}
        loading="eager"
      />
    </div>
  );
};