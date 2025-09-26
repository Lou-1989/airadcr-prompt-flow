import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { 
  Send, 
  Copy, 
  Check, 
  AlertTriangle, 
  Zap,
  Target,
  Clipboard,
  Info
} from 'lucide-react';
import { useTauriCommands } from '@/hooks/useTauriCommands';
import { useToast } from '@/hooks/use-toast';

export const SystemIntegrationPanel = () => {
  const { injectTextToSystem, activeWindow, isTauriApp } = useTauriCommands();
  const { toast } = useToast();
  const [testText, setTestText] = useState('Bonjour, ceci est un test d\'injection automatique depuis AirADCR Desktop !');
  const [isInjecting, setIsInjecting] = useState(false);
  const [lastInjectionResult, setLastInjectionResult] = useState<boolean | null>(null);

  const handleInjectText = async () => {
    if (!testText.trim()) {
      toast({
        title: "Texte requis",
        description: "Veuillez saisir du texte à injecter",
        variant: "destructive"
      });
      return;
    }

    setIsInjecting(true);
    setLastInjectionResult(null);

    try {
      const success = await injectTextToSystem(testText);
      setLastInjectionResult(success);

      if (success) {
        toast({
          title: "✅ Injection réussie",
          description: `Texte injecté dans ${activeWindow?.appName || 'l\'application active'}`,
        });
      } else {
        toast({
          title: "❌ Injection échouée",
          description: "Impossible d'injecter le texte dans l'application cible",
          variant: "destructive"
        });
      }
    } catch (error) {
      console.error('Erreur injection:', error);
      setLastInjectionResult(false);
      toast({
        title: "Erreur système",
        description: "Une erreur est survenue lors de l'injection",
        variant: "destructive"
      });
    } finally {
      setIsInjecting(false);
    }
  };

  const handleCopyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(testText);
      toast({
        title: "📋 Copié",
        description: "Texte copié dans le presse-papiers",
      });
    } catch (error) {
      toast({
        title: "Erreur",
        description: "Impossible de copier le texte",
        variant: "destructive"
      });
    }
  };

  return (
    <div className="p-6 space-y-6 bg-background">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Zap className="w-5 h-5 text-primary" />
            Test d'injection système
          </CardTitle>
          <CardDescription>
            Testez l'injection automatique de texte dans d'autres applications Windows
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Status and Target Info */}
          <div className="space-y-2">
            <Label className="flex items-center gap-2">
              <Target className="w-4 h-4" />
              Application cible
            </Label>
            {activeWindow ? (
              <div className="flex items-center justify-between p-3 bg-secondary/20 rounded-lg border">
                <div>
                  <div className="font-medium">{activeWindow.appName}</div>
                  <div className="text-sm text-muted-foreground">{activeWindow.windowTitle}</div>
                  <div className="text-xs text-muted-foreground">PID: {activeWindow.processId}</div>
                </div>
                <Badge variant={activeWindow.isCompatible ? "secondary" : "outline"}>
                  {activeWindow.isCompatible ? "Compatible" : "Limité"}
                </Badge>
              </div>
            ) : (
              <div className="p-3 bg-muted/20 rounded-lg border-dashed border-2 text-center text-muted-foreground">
                Aucune application détectée
              </div>
            )}
          </div>

          <Separator />

          {/* Text Input */}
          <div className="space-y-2">
            <Label htmlFor="test-text">Texte à injecter</Label>
            <div className="space-y-2">
              <Input
                id="test-text"
                value={testText}
                onChange={(e) => setTestText(e.target.value)}
                placeholder="Saisissez le texte à injecter..."
                className="min-h-[80px]"
              />
              <div className="text-xs text-muted-foreground">
                {testText.length} caractères
              </div>
            </div>
          </div>

          {/* Action Buttons */}
          <div className="flex gap-2">
            <Button 
              onClick={handleInjectText}
              disabled={isInjecting || !activeWindow || !testText.trim()}
              className="flex-1"
            >
              {isInjecting ? (
                <>
                  <div className="animate-spin w-4 h-4 mr-2 border-2 border-current border-t-transparent rounded-full" />
                  Injection...
                </>
              ) : (
                <>
                  <Send className="w-4 h-4 mr-2" />
                  {isTauriApp ? 'Injecter (Natif)' : 'Tester (Demo)'}
                </>
              )}
            </Button>
            
            <Button 
              onClick={handleCopyToClipboard}
              variant="outline"
              disabled={!testText.trim()}
            >
              <Copy className="w-4 h-4 mr-2" />
              Copier
            </Button>
          </div>

          {/* Result Display */}
          {lastInjectionResult !== null && (
            <div className={`p-3 rounded-lg border flex items-center gap-2 ${
              lastInjectionResult 
                ? 'bg-success/10 border-success/30 text-success' 
                : 'bg-destructive/10 border-destructive/30 text-destructive'
            }`}>
              {lastInjectionResult ? (
                <Check className="w-4 h-4" />
              ) : (
                <AlertTriangle className="w-4 h-4" />
              )}
              <span className="text-sm font-medium">
                {lastInjectionResult 
                  ? 'Injection réussie dans l\'application cible'
                  : 'Échec de l\'injection - vérifiez les permissions'
                }
              </span>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Information Card */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Info className="w-5 h-5 text-blue-500" />
            Comment ça fonctionne
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3 text-sm text-muted-foreground">
            <div className="flex items-start gap-3">
              <div className="w-6 h-6 bg-primary/10 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5">
                <span className="text-xs font-bold text-primary">1</span>
              </div>
              <div>
                <strong>Détection automatique:</strong> L'application surveille constamment la fenêtre active du système
              </div>
            </div>
            
            <div className="flex items-start gap-3">
              <div className="w-6 h-6 bg-primary/10 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5">
                <span className="text-xs font-bold text-primary">2</span>
              </div>
              <div>
                <strong>Injection native:</strong> Utilise l'API Windows SendInput pour simuler la saisie clavier
              </div>
            </div>
            
            <div className="flex items-start gap-3">
              <div className="w-6 h-6 bg-primary/10 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5">
                <span className="text-xs font-bold text-primary">3</span>
              </div>
              <div>
                <strong>Presse-papiers intelligent:</strong> Sauvegarde et restaure automatiquement le contenu existant
              </div>
            </div>
          </div>
          
          {!isTauriApp && (
            <div className="mt-4 p-3 bg-warning/10 border border-warning/30 rounded-lg">
              <div className="flex items-center gap-2 text-warning font-medium text-sm">
                <Clipboard className="w-4 h-4" />
                Mode Démonstration
              </div>
              <div className="text-xs mt-1 text-muted-foreground">
                Cette interface montre le fonctionnement prévu. L'injection réelle nécessite l'application Tauri native.
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
};