 import { useState, useEffect, useCallback } from 'react';
 import { Button } from '@/components/ui/button';
 import { Input } from '@/components/ui/input';
 import { Label } from '@/components/ui/label';
 import { Badge } from '@/components/ui/badge';
 import { Switch } from '@/components/ui/switch';
 import { Separator } from '@/components/ui/separator';
 import { Card, CardContent } from '@/components/ui/card';
 import { 
   Server, 
   Wifi, 
   WifiOff, 
   RefreshCw, 
   Shield,
   Clock,
   AlertCircle,
   CheckCircle2,
   Loader2
 } from 'lucide-react';
 import { logger } from '@/utils/logger';
 
interface TeoHubConfigInfo {
  enabled: boolean;
  host: string;
  port: number;
  tls_enabled: boolean;
  timeout_secs: number;
  retry_count: number;
  has_api_token: boolean;
  has_tls_certs: boolean;
}
 
interface TeoHealthResponse {
  ok: boolean;
  service: string;
}
 
 type ConnectionStatus = 'connected' | 'disconnected' | 'checking' | 'disabled' | 'unknown';
 
 interface TeoHubConfigProps {
   isTauriApp: boolean;
 }
 
 export const TeoHubConfig = ({ isTauriApp }: TeoHubConfigProps) => {
   const [config, setConfig] = useState<TeoHubConfigInfo | null>(null);
   const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>('unknown');
   const [healthInfo, setHealthInfo] = useState<TeoHealthResponse | null>(null);
   const [lastCheck, setLastCheck] = useState<Date | null>(null);
   const [error, setError] = useState<string | null>(null);
   const [isLoading, setIsLoading] = useState(false);
 
   // Charger la configuration au montage
   useEffect(() => {
     if (!isTauriApp) return;
     
     const loadConfig = async () => {
       try {
         const { invoke } = await import('@tauri-apps/api/tauri');
         const cfg = await invoke<TeoHubConfigInfo>('teo_get_config');
         setConfig(cfg);
         
         if (!cfg.enabled) {
           setConnectionStatus('disabled');
         }
       } catch (err) {
         logger.error('Erreur chargement config TÉO Hub', err);
         setError('Impossible de charger la configuration');
       }
     };
     
     loadConfig();
   }, [isTauriApp]);
 
   // Vérifier la connexion
   const checkConnection = useCallback(async () => {
     if (!isTauriApp || !config?.enabled) return;
     
     setIsLoading(true);
     setConnectionStatus('checking');
     setError(null);
     
     try {
       const { invoke } = await import('@tauri-apps/api/tauri');
       const health = await invoke<TeoHealthResponse>('teo_check_health');
       
       setHealthInfo(health);
       setConnectionStatus('connected');
       setLastCheck(new Date());
       logger.info('TÉO Hub health check OK', health);
     } catch (err) {
       const errorMsg = err instanceof Error ? err.message : String(err);
       setConnectionStatus('disconnected');
       setError(errorMsg);
       setHealthInfo(null);
       logger.error('TÉO Hub health check failed', err);
     } finally {
       setIsLoading(false);
     }
   }, [isTauriApp, config?.enabled]);
 
   // Auto-check au chargement si activé
   useEffect(() => {
     if (config?.enabled && connectionStatus === 'unknown') {
       checkConnection();
     }
   }, [config?.enabled, connectionStatus, checkConnection]);
 
   const getStatusBadge = () => {
     switch (connectionStatus) {
       case 'connected':
         return (
          <Badge className="bg-primary/20 text-primary border-primary/30">
             <CheckCircle2 className="w-3 h-3 mr-1" />
             Connecté
           </Badge>
         );
       case 'disconnected':
         return (
          <Badge variant="destructive">
             <WifiOff className="w-3 h-3 mr-1" />
             Déconnecté
           </Badge>
         );
       case 'checking':
         return (
          <Badge variant="secondary" className="bg-secondary text-secondary-foreground">
             <Loader2 className="w-3 h-3 mr-1 animate-spin" />
             Vérification...
           </Badge>
         );
       case 'disabled':
         return (
           <Badge variant="secondary" className="bg-muted text-muted-foreground">
             <WifiOff className="w-3 h-3 mr-1" />
             Désactivé
           </Badge>
         );
       default:
         return (
           <Badge variant="outline">
             <AlertCircle className="w-3 h-3 mr-1" />
             Inconnu
           </Badge>
         );
     }
   };
 
   if (!isTauriApp) {
     return (
       <div className="text-center text-muted-foreground py-8">
         <Server className="w-12 h-12 mx-auto mb-2 opacity-50" />
         <p className="text-sm">TÉO Hub Client non disponible</p>
         <p className="text-xs">Nécessite l'application Tauri</p>
       </div>
     );
   }
 
   return (
     <div className="space-y-4">
       {/* Statut de connexion */}
       <Card className="bg-background/50">
         <CardContent className="p-4">
           <div className="flex items-center justify-between mb-3">
             <div className="flex items-center gap-2">
               <Server className="w-4 h-4 text-primary" />
               <span className="font-medium text-sm">TÉO Hub Server</span>
             </div>
             {getStatusBadge()}
           </div>
           
           {config && (
             <div className="text-xs text-muted-foreground space-y-1">
               <div className="flex justify-between">
                 <span>Adresse:</span>
                 <span className="font-mono">
                   {config.tls_enabled ? 'https' : 'http'}://{config.host}:{config.port}
                 </span>
               </div>
                {healthInfo?.service && (
                  <div className="flex justify-between">
                    <span>Service:</span>
                    <span>{healthInfo.service}</span>
                  </div>
                )}
               {lastCheck && (
                 <div className="flex justify-between">
                   <span>Dernière vérif:</span>
                   <span>{lastCheck.toLocaleTimeString()}</span>
                 </div>
               )}
             </div>
           )}
           
           {error && (
             <div className="mt-2 p-2 bg-destructive/10 border border-destructive/20 rounded text-xs text-destructive">
               <AlertCircle className="w-3 h-3 inline mr-1" />
               {error}
             </div>
           )}
           
           <Button
             variant="outline"
             size="sm"
             className="w-full mt-3"
             onClick={checkConnection}
             disabled={isLoading || !config?.enabled}
           >
             <RefreshCw className={`w-3 h-3 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
             Tester la connexion
           </Button>
         </CardContent>
       </Card>
 
       <Separator />
 
       {/* Configuration */}
       <div className="space-y-3">
         <h4 className="text-sm font-semibold flex items-center gap-2">
           <Shield className="w-4 h-4" />
           Configuration
         </h4>
         
         {config && (
           <div className="space-y-2 text-xs">
             <div className="flex items-center justify-between p-2 bg-background/50 rounded">
               <span>Client activé</span>
               <Switch checked={config.enabled} disabled />
             </div>
             
             <div className="flex items-center justify-between p-2 bg-background/50 rounded">
               <span>TLS/SSL</span>
               <Badge variant={config.tls_enabled ? "default" : "secondary"} className="text-xs">
                 {config.tls_enabled ? "Activé" : "Désactivé"}
               </Badge>
             </div>
             
              <div className="flex items-center justify-between p-2 bg-background/50 rounded">
                <span>API Token</span>
                <Badge variant={config.has_api_token ? "default" : "outline"} className="text-xs">
                  {config.has_api_token ? "Configuré" : "Non configuré"}
                </Badge>
              </div>
             
             <div className="flex items-center justify-between p-2 bg-background/50 rounded">
               <span>Certificats mTLS</span>
               <Badge variant={config.has_tls_certs ? "default" : "outline"} className="text-xs">
                 {config.has_tls_certs ? "Présents" : "Absents"}
               </Badge>
             </div>
             
             <div className="flex items-center justify-between p-2 bg-background/50 rounded">
               <span className="flex items-center gap-1">
                 <Clock className="w-3 h-3" />
                 Timeout
               </span>
               <span className="font-mono">{config.timeout_secs}s</span>
             </div>
             
             <div className="flex items-center justify-between p-2 bg-background/50 rounded">
               <span>Retries</span>
               <span className="font-mono">{config.retry_count}x</span>
             </div>
           </div>
         )}
       </div>
 
       <Separator />
 
       {/* Instructions */}
       <div className="text-xs text-muted-foreground space-y-2">
         <p className="font-medium">Configuration via config.toml:</p>
        <pre className="p-2 bg-muted/50 rounded text-[10px] overflow-x-auto">
{`[teo_hub]
enabled = true
host = "192.168.1.253"
port = 54489
api_token = "votre_token"`}
        </pre>
         <p className="text-[10px]">
           Chemin: <code className="bg-muted px-1 rounded">%APPDATA%/airadcr-desktop/config.toml</code>
         </p>
       </div>
     </div>
   );
 };