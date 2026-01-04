import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { supabase } from "@/integrations/supabase/client";
import { toast } from "sonner";
import { PricingCard } from "@/components/pricing/PricingCard";
import { BillingToggle } from "@/components/pricing/BillingToggle";
import { B2BForm } from "@/components/pricing/B2BForm";
import { SUBSCRIPTION_PLANS, type PlanType, type BillingInterval } from "@/config/subscriptionPlans";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { Building2, User } from "lucide-react";

interface BusinessInfo {
  companyName: string;
  siret: string;
  vatNumber: string;
}

export default function Pricing() {
  const navigate = useNavigate();
  const [isAnnual, setIsAnnual] = useState(false);
  const [loadingPlan, setLoadingPlan] = useState<string | null>(null);
  const [isB2B, setIsB2B] = useState(false);
  const [businessInfo, setBusinessInfo] = useState<BusinessInfo>({
    companyName: "",
    siret: "",
    vatNumber: ""
  });
  const [formErrors, setFormErrors] = useState<Partial<Record<keyof BusinessInfo, string>>>({});

  const interval: BillingInterval = isAnnual ? "year" : "month";

  const validateB2BForm = (): boolean => {
    const errors: Partial<Record<keyof BusinessInfo, string>> = {};
    
    if (isB2B) {
      if (!businessInfo.companyName.trim()) {
        errors.companyName = "La raison sociale est requise";
      } else if (businessInfo.companyName.trim().length < 2) {
        errors.companyName = "La raison sociale doit contenir au moins 2 caractères";
      }
    }
    
    setFormErrors(errors);
    return Object.keys(errors).length === 0;
  };

  const handleSelectPlan = async (planKey: string) => {
    if (planKey === "equipes") {
      const subject = encodeURIComponent("Demande de devis - Plan Équipes");
      window.location.href = `mailto:contact@airadcr.com?subject=${subject}`;
      return;
    }

    if (!validateB2BForm()) {
      toast.error("Veuillez corriger les erreurs du formulaire");
      return;
    }

    setLoadingPlan(planKey);
    
    try {
      const { data: { session } } = await supabase.auth.getSession();
      
      if (!session) {
        toast.error("Veuillez vous connecter pour souscrire à un abonnement");
        navigate("/auth");
        return;
      }

      const plan = SUBSCRIPTION_PLANS[planKey as PlanType];
      const priceId = plan.prices[interval].id;

      const { data, error } = await supabase.functions.invoke("create-checkout-session", {
        body: { 
          priceId,
          customerType: isB2B ? "business" : "individual",
          businessInfo: isB2B ? {
            companyName: businessInfo.companyName.trim(),
            siret: businessInfo.siret.trim(),
            vatNumber: businessInfo.vatNumber.trim()
          } : undefined
        }
      });

      if (error) throw error;
      if (data?.url) {
        window.open(data.url, "_blank");
      }
    } catch (error) {
      console.error("Checkout error:", error);
      toast.error("Erreur lors de la création de la session de paiement");
    } finally {
      setLoadingPlan(null);
    }
  };

  return (
    <div className="min-h-screen bg-background py-16 px-4">
      <div className="max-w-6xl mx-auto">
        <div className="text-center mb-12">
          <h1 className="text-4xl font-bold text-foreground mb-4">
            Choisissez votre plan
          </h1>
          <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
            Dictée vocale médicale professionnelle avec ou sans structuration IA
          </p>
        </div>

        {/* Toggle B2B */}
        <div className="flex items-center justify-center gap-4 mb-6">
          <div className={`flex items-center gap-2 ${!isB2B ? 'text-foreground font-medium' : 'text-muted-foreground'}`}>
            <User className="h-4 w-4" />
            <Label htmlFor="b2b-toggle" className="cursor-pointer">Particulier</Label>
          </div>
          <Switch
            id="b2b-toggle"
            checked={isB2B}
            onCheckedChange={setIsB2B}
          />
          <div className={`flex items-center gap-2 ${isB2B ? 'text-foreground font-medium' : 'text-muted-foreground'}`}>
            <Building2 className="h-4 w-4" />
            <Label htmlFor="b2b-toggle" className="cursor-pointer">Professionnel</Label>
          </div>
        </div>

        <BillingToggle isAnnual={isAnnual} onToggle={setIsAnnual} />

        {/* B2B Form */}
        {isB2B && (
          <div className="max-w-md mx-auto mb-8">
            <B2BForm 
              businessInfo={businessInfo} 
              onChange={setBusinessInfo}
              errors={formErrors}
            />
          </div>
        )}

        <div className="grid md:grid-cols-3 gap-8 mt-8">
          {Object.entries(SUBSCRIPTION_PLANS).map(([key, plan]) => {
            const hasPrices = 'prices' in plan;
            const priceData = hasPrices ? plan.prices[interval] : null;
            const savings = priceData && 'savings' in priceData ? priceData.savings : undefined;
            
            return (
              <PricingCard
                key={key}
                name={plan.name}
                description={plan.description}
                price={priceData ? priceData.amount : null}
                interval={interval}
                savings={savings}
                features={[...plan.features]}
                popular={'popular' in plan ? plan.popular : false}
                contactOnly={'contactOnly' in plan ? plan.contactOnly : false}
                onSelect={() => handleSelectPlan(key)}
                isLoading={loadingPlan === key}
              />
            );
          })}
        </div>

        <div className="mt-16 text-center text-sm text-muted-foreground">
          <p>Tous les prix sont en CHF. TVA non applicable (art. 293 B du CGI).</p>
          <p className="mt-2">Annulation possible à tout moment depuis votre espace client.</p>
        </div>
      </div>
    </div>
  );
}
