import { useEffect } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { CheckCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

export default function SubscriptionSuccess() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const sessionId = searchParams.get("session_id");

  useEffect(() => {
    if (sessionId) {
      console.log("Checkout session completed:", sessionId);
    }
  }, [sessionId]);

  return (
    <div className="min-h-screen bg-background flex items-center justify-center p-4">
      <Card className="max-w-md w-full text-center">
        <CardHeader>
          <div className="mx-auto mb-4">
            <CheckCircle className="h-16 w-16 text-green-500" />
          </div>
          <CardTitle className="text-2xl">Abonnement activé !</CardTitle>
          <CardDescription>
            Merci pour votre confiance. Votre abonnement est maintenant actif.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-sm text-muted-foreground">
            Vous pouvez maintenant accéder à toutes les fonctionnalités de votre plan.
            Un email de confirmation vous a été envoyé.
          </p>
          <div className="flex flex-col gap-2">
            <Button onClick={() => navigate("/")} className="w-full">
              Accéder à l'application
            </Button>
            <Button variant="outline" onClick={() => navigate("/account")} className="w-full">
              Voir mon compte
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
