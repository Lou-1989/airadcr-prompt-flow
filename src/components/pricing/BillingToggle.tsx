import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";

interface BillingToggleProps {
  isAnnual: boolean;
  onToggle: (annual: boolean) => void;
}

export function BillingToggle({ isAnnual, onToggle }: BillingToggleProps) {
  return (
    <div className="flex items-center justify-center gap-4 py-6">
      <Label 
        htmlFor="billing-toggle" 
        className={`text-sm cursor-pointer transition-colors ${!isAnnual ? 'text-foreground font-medium' : 'text-muted-foreground'}`}
      >
        Mensuel
      </Label>
      <Switch
        id="billing-toggle"
        checked={isAnnual}
        onCheckedChange={onToggle}
      />
      <div className="flex items-center gap-2">
        <Label 
          htmlFor="billing-toggle" 
          className={`text-sm cursor-pointer transition-colors ${isAnnual ? 'text-foreground font-medium' : 'text-muted-foreground'}`}
        >
          Annuel
        </Label>
        {isAnnual && (
          <Badge variant="secondary" className="text-xs">
            Économisez jusqu'à 20%
          </Badge>
        )}
      </div>
    </div>
  );
}
