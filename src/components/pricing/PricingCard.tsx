import { Check } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

interface PricingCardProps {
  name: string;
  description: string;
  price: number | null;
  interval: "month" | "year";
  savings?: string;
  features: string[];
  popular?: boolean;
  contactOnly?: boolean;
  onSelect: () => void;
  isLoading?: boolean;
}

export function PricingCard({
  name,
  description,
  price,
  interval,
  savings,
  features,
  popular,
  contactOnly,
  onSelect,
  isLoading
}: PricingCardProps) {
  return (
    <Card className={cn(
      "relative flex flex-col transition-all duration-300 hover:shadow-lg",
      popular && "border-primary shadow-md scale-105"
    )}>
      {popular && (
        <Badge className="absolute -top-3 left-1/2 -translate-x-1/2 bg-primary text-primary-foreground">
          Populaire
        </Badge>
      )}
      
      <CardHeader className="text-center pb-2">
        <CardTitle className="text-2xl font-bold">{name}</CardTitle>
        <CardDescription className="text-sm">{description}</CardDescription>
      </CardHeader>
      
      <CardContent className="flex-1">
        <div className="text-center mb-6">
          {contactOnly ? (
            <div className="text-3xl font-bold text-foreground">Sur devis</div>
          ) : (
            <>
              <div className="flex items-baseline justify-center gap-1">
                <span className="text-4xl font-bold text-foreground">{price}</span>
                <span className="text-muted-foreground">CHF</span>
              </div>
              <div className="text-sm text-muted-foreground">
                {interval === "month" ? "/mois" : "/an"}
                {savings && (
                  <Badge variant="secondary" className="ml-2">
                    -{savings}
                  </Badge>
                )}
              </div>
            </>
          )}
        </div>
        
        <ul className="space-y-3">
          {features.map((feature, index) => (
            <li key={index} className="flex items-start gap-2">
              <Check className="h-5 w-5 text-primary shrink-0 mt-0.5" />
              <span className="text-sm text-muted-foreground">{feature}</span>
            </li>
          ))}
        </ul>
      </CardContent>
      
      <CardFooter>
        <Button 
          className="w-full" 
          variant={popular ? "default" : "outline"}
          onClick={onSelect}
          disabled={isLoading}
        >
          {isLoading ? "Chargement..." : contactOnly ? "Nous contacter" : "Choisir ce plan"}
        </Button>
      </CardFooter>
    </Card>
  );
}
