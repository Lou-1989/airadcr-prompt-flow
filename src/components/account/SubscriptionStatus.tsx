import { useState, useEffect } from "react";
import { supabase } from "@/integrations/supabase/client";
import { toast } from "sonner";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Loader2, CreditCard, Calendar, ExternalLink } from "lucide-react";
import { SUBSCRIPTION_PLANS } from "@/config/subscriptionPlans";

interface SubscriptionData {
  subscribed: boolean;
  plan_type?: string;
  billing_interval?: string;
  subscription_end?: string;
  cancel_at_period_end?: boolean;
}

export function SubscriptionStatus() {
  const [subscription, setSubscription] = useState<SubscriptionData | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isPortalLoading, setIsPortalLoading] = useState(false);

  const fetchSubscription = async () => {
    try {
      const { data: { session } } = await supabase.auth.getSession();
      if (!session) {
        setSubscription({ subscribed: false });
        setIsLoading(false);
        return;
      }

      const { data, error } = await supabase.functions.invoke("check-subscription");
      
      if (error) throw error;
      setSubscription(data);
    } catch (error) {
      console.error("Error fetching subscription:", error);
      setSubscription({ subscribed: false });
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchSubscription();
    
    // Auto-refresh every 60 seconds
    const interval = setInterval(fetchSubscription, 60000);
    return () => clearInterval(interval);
  }, []);

  const handleManageSubscription = async () => {
    setIsPortalLoading(true);
    try {
      const { data, error } = await supabase.functions.invoke("customer-portal");
      
      if (error) throw error;
      if (data?.url) {
        window.open(data.url, "_blank");
      }
    } catch (error) {
      console.error("Portal error:", error);
      toast.error("Erreur lors de l'ouverture du portail de gestion");
    } finally {
      setIsPortalLoading(false);
    }
  };

  if (isLoading) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center py-8">
          <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
        </CardContent>
      </Card>
    );
  }

  const planInfo = subscription?.plan_type 
    ? SUBSCRIPTION_PLANS[subscription.plan_type as keyof typeof SUBSCRIPTION_PLANS]
    : null;

  const formatDate = (dateString?: string) => {
    if (!dateString) return "";
    return new Date(dateString).toLocaleDateString("fr-FR", {
      day: "numeric",
      month: "long",
      year: "numeric"
    });
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <CreditCard className="h-5 w-5" />
          Mon abonnement
        </CardTitle>
        <CardDescription>
          Gérez votre abonnement et vos factures
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {subscription?.subscribed && planInfo ? (
          <>
            <div className="flex items-center justify-between">
              <div>
                <div className="flex items-center gap-2">
                  <span className="font-semibold text-lg">{planInfo.name}</span>
                  <Badge variant="default">Actif</Badge>
                  {subscription.cancel_at_period_end && (
                    <Badge variant="destructive">Annulation prévue</Badge>
                  )}
                </div>
                <p className="text-sm text-muted-foreground mt-1">
                  {planInfo.description}
                </p>
              </div>
            </div>

            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Calendar className="h-4 w-4" />
              <span>
                {subscription.cancel_at_period_end 
                  ? `Se termine le ${formatDate(subscription.subscription_end)}`
                  : `Prochain renouvellement : ${formatDate(subscription.subscription_end)}`
                }
              </span>
            </div>

            <div className="text-sm text-muted-foreground">
              Facturation : {subscription.billing_interval === "year" ? "Annuelle" : "Mensuelle"}
            </div>

            <Button 
              onClick={handleManageSubscription} 
              disabled={isPortalLoading}
              className="w-full"
            >
              {isPortalLoading ? (
                <Loader2 className="h-4 w-4 animate-spin mr-2" />
              ) : (
                <ExternalLink className="h-4 w-4 mr-2" />
              )}
              Gérer mon abonnement
            </Button>
          </>
        ) : (
          <div className="text-center py-4">
            <p className="text-muted-foreground mb-4">
              Vous n'avez pas d'abonnement actif
            </p>
            <Button asChild>
              <a href="/pricing">Voir les plans</a>
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
