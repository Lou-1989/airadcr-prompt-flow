import { useState, useEffect, useCallback } from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { invoke } from '@tauri-apps/api/tauri';
import { RefreshCw, Trash2, Database, Key, FileText, X, Plus, Copy, Check, Ban } from 'lucide-react';

interface DatabaseStats {
  total_reports: number;
  pending_reports: number;
  retrieved_reports: number;
  expired_reports: number;
  active_api_keys: number;
  revoked_api_keys: number;
}

interface PendingReportSummary {
  technical_id: string;
  accession_number: string | null;
  patient_id: string | null;
  modality: string | null;
  status: string;
  source_type: string;
  created_at: string;
  expires_at: string;
}

interface ApiKeySummary {
  id: string;
  name: string;
  key_prefix: string;
  is_active: boolean;
  created_at: string;
}

interface NewApiKeyResult {
  key_prefix: string;
  full_key: string;
  name: string;
}

interface DatabaseTabProps {
  isTauriApp: boolean;
}

export const DatabaseTab = ({ isTauriApp }: DatabaseTabProps) => {
  const [stats, setStats] = useState<DatabaseStats | null>(null);
  const [reports, setReports] = useState<PendingReportSummary[]>([]);
  const [apiKeys, setApiKeys] = useState<ApiKeySummary[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [reportToDelete, setReportToDelete] = useState<PendingReportSummary | null>(null);
  
  // États pour la création de clé API
  const [showCreateKeyDialog, setShowCreateKeyDialog] = useState(false);
  const [newKeyName, setNewKeyName] = useState('');
  const [createdKey, setCreatedKey] = useState<NewApiKeyResult | null>(null);
  const [isCreatingKey, setIsCreatingKey] = useState(false);
  const [keyCopied, setKeyCopied] = useState(false);
  
  // État pour la révocation
  const [keyToRevoke, setKeyToRevoke] = useState<ApiKeySummary | null>(null);

  const fetchData = useCallback(async () => {
    if (!isTauriApp) return;
    
    setIsLoading(true);
    setError(null);
    
    try {
      const [statsResult, reportsResult, keysResult] = await Promise.all([
        invoke<DatabaseStats>('get_database_stats'),
        invoke<PendingReportSummary[]>('get_all_pending_reports'),
        invoke<ApiKeySummary[]>('get_api_keys_list'),
      ]);
      
      setStats(statsResult);
      setReports(reportsResult);
      setApiKeys(keysResult);
      setLastUpdate(new Date());
    } catch (err) {
      console.error('Erreur chargement données DB:', err);
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }, [isTauriApp]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  const handleCleanup = async () => {
    if (!isTauriApp) return;
    
    try {
      const count = await invoke<number>('cleanup_expired_reports_cmd');
      console.log(`Cleanup: ${count} rapports supprimés`);
      await fetchData();
    } catch (err) {
      console.error('Erreur cleanup:', err);
      setError(String(err));
    }
  };

  const handleDeleteReport = async () => {
    if (!isTauriApp || !reportToDelete) return;
    
    try {
      const deleted = await invoke<boolean>('delete_pending_report_cmd', { technicalId: reportToDelete.technical_id });
      if (deleted) {
        console.log(`Rapport ${reportToDelete.technical_id} supprimé`);
        await fetchData();
      }
    } catch (err) {
      console.error('Erreur suppression rapport:', err);
      setError(String(err));
    } finally {
      setReportToDelete(null);
    }
  };

  const confirmDeleteReport = (report: PendingReportSummary) => {
    setReportToDelete(report);
  };

  // Création de clé API
  const handleCreateApiKey = async () => {
    if (!isTauriApp || !newKeyName.trim()) return;
    
    setIsCreatingKey(true);
    try {
      const result = await invoke<NewApiKeyResult>('create_api_key_cmd', { name: newKeyName.trim() });
      setCreatedKey(result);
      await fetchData();
    } catch (err) {
      console.error('Erreur création clé API:', err);
      setError(String(err));
    } finally {
      setIsCreatingKey(false);
    }
  };

  const handleCopyKey = async () => {
    if (createdKey) {
      await navigator.clipboard.writeText(createdKey.full_key);
      setKeyCopied(true);
      setTimeout(() => setKeyCopied(false), 2000);
    }
  };

  const handleCloseCreateDialog = () => {
    setShowCreateKeyDialog(false);
    setNewKeyName('');
    setCreatedKey(null);
    setKeyCopied(false);
  };

  // Révocation de clé API
  const handleRevokeApiKey = async () => {
    if (!isTauriApp || !keyToRevoke) return;
    
    try {
      await invoke<boolean>('revoke_api_key_cmd', { keyPrefix: keyToRevoke.key_prefix });
      await fetchData();
    } catch (err) {
      console.error('Erreur révocation clé API:', err);
      setError(String(err));
    } finally {
      setKeyToRevoke(null);
    }
  };

  const formatDate = (dateStr: string) => {
    try {
      const date = new Date(dateStr);
      return date.toLocaleString('fr-FR', {
        day: '2-digit',
        month: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return dateStr;
    }
  };

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'pending':
        return <Badge variant="outline" className="bg-yellow-500/10 text-yellow-600 border-yellow-500/30">⏳ En attente</Badge>;
      case 'retrieved':
        return <Badge variant="outline" className="bg-green-500/10 text-green-600 border-green-500/30">✅ Récupéré</Badge>;
      case 'expired':
        return <Badge variant="outline" className="bg-red-500/10 text-red-600 border-red-500/30">❌ Expiré</Badge>;
      default:
        return <Badge variant="secondary">{status}</Badge>;
    }
  };

  if (!isTauriApp) {
    return (
      <div className="p-4 text-center text-muted-foreground">
        <Database className="w-8 h-8 mx-auto mb-2 opacity-50" />
        <p className="text-sm">Base de données non disponible en mode web</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Header avec boutons */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Database className="w-4 h-4" />
          <span className="text-sm font-medium">Base SQLite</span>
        </div>
        <div className="flex gap-1">
          <Button
            size="sm"
            variant="ghost"
            onClick={handleCleanup}
            disabled={isLoading}
            className="h-7 px-2"
            title="Nettoyer les rapports expirés"
          >
            <Trash2 className="w-3 h-3" />
          </Button>
          <Button
            size="sm"
            variant="ghost"
            onClick={fetchData}
            disabled={isLoading}
            className="h-7 px-2"
            title="Rafraîchir"
          >
            <RefreshCw className={`w-3 h-3 ${isLoading ? 'animate-spin' : ''}`} />
          </Button>
        </div>
      </div>

      {error && (
        <div className="p-2 text-xs text-red-500 bg-red-500/10 rounded">
          {error}
        </div>
      )}

      {/* Statistiques */}
      {stats && (
        <div className="grid grid-cols-2 gap-2">
          <div className="p-2 bg-background/50 rounded text-center">
            <div className="text-lg font-bold text-primary">{stats.pending_reports}</div>
            <div className="text-[10px] text-muted-foreground">En attente</div>
          </div>
          <div className="p-2 bg-background/50 rounded text-center">
            <div className="text-lg font-bold text-green-500">{stats.retrieved_reports}</div>
            <div className="text-[10px] text-muted-foreground">Récupérés</div>
          </div>
          <div className="p-2 bg-background/50 rounded text-center">
            <div className="text-lg font-bold text-muted-foreground">{stats.expired_reports}</div>
            <div className="text-[10px] text-muted-foreground">Expirés</div>
          </div>
          <div className="p-2 bg-background/50 rounded text-center">
            <div className="text-lg font-bold text-blue-500">{stats.active_api_keys}</div>
            <div className="text-[10px] text-muted-foreground">Clés API</div>
          </div>
        </div>
      )}

      <Separator />

      {/* Liste des rapports */}
      <div className="space-y-2">
        <div className="flex items-center gap-2">
          <FileText className="w-3 h-3" />
          <span className="text-xs font-medium">Rapports ({reports.length})</span>
        </div>
        
        <ScrollArea className="h-32">
          {reports.length === 0 ? (
            <div className="text-xs text-muted-foreground text-center py-4">
              Aucun rapport en base
            </div>
          ) : (
            <div className="space-y-1">
              {reports.map((report) => (
                <div
                  key={report.technical_id}
                  className="p-2 bg-background/50 rounded text-xs space-y-1 group relative"
                >
                  <div className="flex items-center justify-between">
                    <code className="font-mono text-[10px] text-primary">
                      {report.technical_id.substring(0, 12)}...
                    </code>
                    <div className="flex items-center gap-1">
                      {getStatusBadge(report.status)}
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => confirmDeleteReport(report)}
                        className="h-5 w-5 p-0 opacity-0 group-hover:opacity-100 transition-opacity text-destructive hover:text-destructive hover:bg-destructive/10"
                        title="Supprimer ce rapport"
                      >
                        <X className="w-3 h-3" />
                      </Button>
                    </div>
                  </div>
                  <div className="flex gap-2 text-[10px] text-muted-foreground">
                    {report.accession_number && (
                      <span>ACC: {report.accession_number}</span>
                    )}
                    {report.modality && (
                      <span className="font-medium">{report.modality}</span>
                    )}
                  </div>
                  <div className="text-[10px] text-muted-foreground">
                    {formatDate(report.created_at)} → {formatDate(report.expires_at)}
                  </div>
                </div>
              ))}
            </div>
          )}
        </ScrollArea>
      </div>

      <Separator />

      {/* Liste des clés API */}
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Key className="w-3 h-3" />
            <span className="text-xs font-medium">Clés API ({apiKeys.length})</span>
          </div>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => setShowCreateKeyDialog(true)}
            className="h-6 px-2"
            title="Créer une clé API"
          >
            <Plus className="w-3 h-3" />
          </Button>
        </div>
        
        {apiKeys.length === 0 ? (
          <div className="text-xs text-muted-foreground text-center py-2">
            Aucune clé API configurée
          </div>
        ) : (
          <div className="space-y-1">
            {apiKeys.map((key) => (
              <div
                key={key.id}
                className="p-2 bg-background/50 rounded text-xs flex items-center justify-between group"
              >
                <div>
                  <div className="font-medium">{key.name}</div>
                  <code className="text-[10px] text-muted-foreground">{key.key_prefix}***</code>
                </div>
                <div className="flex items-center gap-1">
                  <Badge variant={key.is_active ? "default" : "secondary"}>
                    {key.is_active ? "Active" : "Révoquée"}
                  </Badge>
                  {key.is_active && (
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => setKeyToRevoke(key)}
                      className="h-5 w-5 p-0 opacity-0 group-hover:opacity-100 transition-opacity text-destructive hover:text-destructive hover:bg-destructive/10"
                      title="Révoquer cette clé"
                    >
                      <Ban className="w-3 h-3" />
                    </Button>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Dernière mise à jour */}
      {lastUpdate && (
        <div className="text-[10px] text-muted-foreground text-center">
          Mis à jour: {lastUpdate.toLocaleTimeString('fr-FR')}
        </div>
      )}

      {/* Dialog de confirmation de suppression */}
      <AlertDialog open={!!reportToDelete} onOpenChange={(open) => !open && setReportToDelete(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Confirmer la suppression</AlertDialogTitle>
            <AlertDialogDescription>
              Voulez-vous vraiment supprimer ce rapport ?
              {reportToDelete && (
                <div className="mt-2 p-2 bg-muted rounded text-xs space-y-1">
                  <div><strong>ID:</strong> {reportToDelete.technical_id.substring(0, 20)}...</div>
                  {reportToDelete.accession_number && (
                    <div><strong>Accession:</strong> {reportToDelete.accession_number}</div>
                  )}
                  {reportToDelete.modality && (
                    <div><strong>Modalité:</strong> {reportToDelete.modality}</div>
                  )}
                </div>
              )}
              <p className="mt-2 text-destructive font-medium">Cette action est irréversible.</p>
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Annuler</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDeleteReport}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              Supprimer
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Dialog de création de clé API */}
      <Dialog open={showCreateKeyDialog} onOpenChange={(open) => !open && handleCloseCreateDialog()}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>
              {createdKey ? '🔑 Clé API créée' : 'Créer une clé API'}
            </DialogTitle>
            <DialogDescription>
              {createdKey 
                ? 'Copiez cette clé maintenant. Elle ne sera plus affichée.'
                : 'Entrez un nom pour identifier cette clé API.'
              }
            </DialogDescription>
          </DialogHeader>
          
          {!createdKey ? (
            <div className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="keyName">Nom de la clé</Label>
                <Input
                  id="keyName"
                  placeholder="Ex: Production TÉO Hub"
                  value={newKeyName}
                  onChange={(e) => setNewKeyName(e.target.value)}
                  maxLength={50}
                />
              </div>
            </div>
          ) : (
            <div className="space-y-4">
              <div className="space-y-2">
                <Label>Nom</Label>
                <div className="text-sm font-medium">{createdKey.name}</div>
              </div>
              <div className="space-y-2">
                <Label>Clé complète</Label>
                <div className="flex items-center gap-2">
                  <code className="flex-1 p-2 bg-muted rounded text-xs break-all">
                    {createdKey.full_key}
                  </code>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={handleCopyKey}
                    className="shrink-0"
                  >
                    {keyCopied ? <Check className="w-4 h-4 text-green-500" /> : <Copy className="w-4 h-4" />}
                  </Button>
                </div>
              </div>
              <div className="p-2 bg-yellow-500/10 border border-yellow-500/30 rounded text-xs text-yellow-700">
                ⚠️ Cette clé ne sera plus affichée après fermeture. Conservez-la en lieu sûr.
              </div>
            </div>
          )}
          
          <DialogFooter>
            {!createdKey ? (
              <>
                <Button variant="outline" onClick={handleCloseCreateDialog}>
                  Annuler
                </Button>
                <Button 
                  onClick={handleCreateApiKey} 
                  disabled={!newKeyName.trim() || isCreatingKey}
                >
                  {isCreatingKey ? 'Création...' : 'Créer'}
                </Button>
              </>
            ) : (
              <Button onClick={handleCloseCreateDialog}>
                Fermer
              </Button>
            )}
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Dialog de confirmation de révocation */}
      <AlertDialog open={!!keyToRevoke} onOpenChange={(open) => !open && setKeyToRevoke(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Révoquer la clé API</AlertDialogTitle>
            <AlertDialogDescription>
              Voulez-vous vraiment révoquer cette clé API ?
              {keyToRevoke && (
                <div className="mt-2 p-2 bg-muted rounded text-xs space-y-1">
                  <div><strong>Nom:</strong> {keyToRevoke.name}</div>
                  <div><strong>Préfixe:</strong> {keyToRevoke.key_prefix}</div>
                </div>
              )}
              <p className="mt-2 text-destructive font-medium">
                Les applications utilisant cette clé ne pourront plus accéder à l'API.
              </p>
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Annuler</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleRevokeApiKey}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              Révoquer
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
};
