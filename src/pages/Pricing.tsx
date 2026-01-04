import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { supabase } from "@/integrations/supabase/client";
import { toast } from "sonner";
import { PricingCard } from "@/components/pricing/PricingCard";
import { BillingToggle } from "@/components/pricing/BillingToggle";
import { SUBSCRIPTION_PLANS, type PlanType, type BillingInterval } from "@/config/subscriptionPlans";

export default function Pricing() {
  const navigate = useNavigate();
  const [isAnnual, setIsAnnual] = useState(false);
  const [loadingPlan, setLoadingPlan] = useState<string | null>(null);

  const interval: BillingInterval = isAnnual ? "year" : "month";

  const handleSelectPlan = async (planKey: string) => {
    if (planKey === "equipes") {
      window.location.href = "mailto:contact@airadcr.com?subject=Demande%20de%20devis%20-%20Plan%20Équipes";
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
          customerType: "individual"
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

        <BillingToggle isAnnual={isAnnual} onToggle={setIsAnnual} />

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
