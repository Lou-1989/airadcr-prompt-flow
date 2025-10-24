import { Download as DownloadIcon, Shield, CheckCircle, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';

const Download = () => {
  const installerUrl = '/downloads/AIRADCR_1.0.0_x64-setup.exe';
  const installerSize = '~12-15 MB';
  const version = '1.0.0';

  return (
    <div className="min-h-screen bg-gradient-to-b from-background to-muted/20 p-6">
      <div className="max-w-4xl mx-auto space-y-8">
        {/* Header */}
        <div className="text-center space-y-4 pt-8">
          <h1 className="text-4xl font-bold">Télécharger AIRADCR Desktop</h1>
          <p className="text-xl text-muted-foreground">
            Application de dictée radiologique professionnelle
          </p>
        </div>

        {/* Download Card */}
        <Card className="border-2">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <DownloadIcon className="h-6 w-6" />
              Windows Installer (64-bit)
            </CardTitle>
            <CardDescription>
              Version {version} • {installerSize} • Windows 10/11
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <Button 
              size="lg" 
              className="w-full text-lg h-14"
              onClick={() => window.location.href = installerUrl}
            >
              <DownloadIcon className="mr-2 h-5 w-5" />
              Télécharger AIRADCR_1.0.0_x64-setup.exe
            </Button>

            <div className="grid gap-3 text-sm">
              <div className="flex items-start gap-2">
                <CheckCircle className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                <span>Téléchargement direct via HTTPS</span>
              </div>
              <div className="flex items-start gap-2">
                <CheckCircle className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                <span>Hébergé officiellement sur airadcr.com</span>
              </div>
              <div className="flex items-start gap-2">
                <CheckCircle className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                <span>Aucun intermédiaire tiers</span>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Security Warning */}
        <Alert>
          <Shield className="h-4 w-4" />
          <AlertTitle>⚠️ Avertissement Windows Defender (Faux Positif)</AlertTitle>
          <AlertDescription className="space-y-2 mt-2">
            <p>
              Windows Defender peut bloquer AIRADCR car l'application utilise des API Windows 
              légitimes pour injecter du texte médical dans vos logiciels RIS/PACS/Word.
            </p>
            <p className="font-semibold">
              C'est un FAUX POSITIF confirmé : notre code est open-source et auditable sur GitHub.
            </p>
          </AlertDescription>
        </Alert>

        {/* Installation Instructions */}
        <Card>
          <CardHeader>
            <CardTitle>Instructions d'Installation</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-3">
              <div className="flex gap-3">
                <div className="flex-shrink-0 w-6 h-6 rounded-full bg-primary text-primary-foreground flex items-center justify-center text-sm font-bold">
                  1
                </div>
                <div>
                  <p className="font-medium">Télécharger le fichier</p>
                  <p className="text-sm text-muted-foreground">Cliquez sur le bouton ci-dessus</p>
                </div>
              </div>

              <div className="flex gap-3">
                <div className="flex-shrink-0 w-6 h-6 rounded-full bg-primary text-primary-foreground flex items-center justify-center text-sm font-bold">
                  2
                </div>
                <div>
                  <p className="font-medium">Exécuter l'installateur</p>
                  <p className="text-sm text-muted-foreground">Double-cliquez sur le fichier téléchargé</p>
                </div>
              </div>

              <div className="flex gap-3">
                <div className="flex-shrink-0 w-6 h-6 rounded-full bg-primary text-primary-foreground flex items-center justify-center text-sm font-bold">
                  3
                </div>
                <div>
                  <p className="font-medium">Si Windows SmartScreen bloque</p>
                  <p className="text-sm text-muted-foreground">
                    Cliquez sur "Informations complémentaires" puis "Exécuter quand même"
                  </p>
                </div>
              </div>

              <div className="flex gap-3">
                <div className="flex-shrink-0 w-6 h-6 rounded-full bg-primary text-primary-foreground flex items-center justify-center text-sm font-bold">
                  4
                </div>
                <div>
                  <p className="font-medium">Suivre l'assistant d'installation</p>
                  <p className="text-sm text-muted-foreground">L'installation prend moins d'une minute</p>
                </div>
              </div>
            </div>

            <Alert variant="destructive" className="mt-4">
              <AlertCircle className="h-4 w-4" />
              <AlertTitle>En cas de blocage par Windows Defender</AlertTitle>
              <AlertDescription>
                Consultez le <a href="/INSTALLATION_GUIDE.md" className="underline font-medium">Guide d'Installation Complet</a> pour 
                ajouter manuellement une exclusion dans Windows Security.
              </AlertDescription>
            </Alert>
          </CardContent>
        </Card>

        {/* Features */}
        <Card>
          <CardHeader>
            <CardTitle>Fonctionnalités Principales</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid gap-3 text-sm">
              <div className="flex items-start gap-2">
                <CheckCircle className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                <span>Injection intelligente de texte dans RIS/PACS/Word</span>
              </div>
              <div className="flex items-start gap-2">
                <CheckCircle className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                <span>Support multi-écrans pour workflows radiologiques</span>
              </div>
              <div className="flex items-start gap-2">
                <CheckCircle className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                <span>Fenêtre toujours au premier plan (always-on-top)</span>
              </div>
              <div className="flex items-start gap-2">
                <CheckCircle className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                <span>Intégration SpeechMike Philips</span>
              </div>
              <div className="flex items-start gap-2">
                <CheckCircle className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                <span>Traitement 100% local (aucune donnée envoyée sur Internet)</span>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Support */}
        <div className="text-center text-sm text-muted-foreground space-y-2 pb-8">
          <p>
            <strong>Support :</strong> contact@airadcr.com • 
            <a href="https://docs.airadcr.com" className="underline ml-1">Documentation</a> • 
            <a href="https://github.com/airadcr/desktop" className="underline ml-1">GitHub</a>
          </p>
          <p className="text-xs">
            Développé par Dr Lounes BENSID - AIRADCR © 2025
          </p>
        </div>
      </div>
    </div>
  );
};

export default Download;
