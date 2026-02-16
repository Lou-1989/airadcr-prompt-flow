import { useState, useCallback } from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { Play, CheckCircle2, XCircle, Loader2, FlaskConical } from 'lucide-react';
import { logger } from '@/utils/logger';

const TEST_DATASETS = [
  {
    id: 'mammo-birads1',
    label: 'Mammographie BI-RADS 1',
    technical_id: 'TEST-MAMMO-001',
    patient_id: '11601',
    accession_number: 'ACC2024TEST01',
    study_instance_uid: '5.1.600.598386.1.618.1.905951.1.945557.148',
    modality: 'MG',
    structured: {
      title: 'Mammographie bilaterale',
      indication: 'Depistage organise',
      technique: 'Incidences CC et MLO bilaterales',
      results: 'Sein droit: parenchyme de densite ACR b. Pas de masse, distorsion ou foyer de microcalcifications. BI-RADS 1.\nSein gauche: parenchyme de densite ACR b. Pas de masse, distorsion ou foyer de microcalcifications. BI-RADS 1.',
      conclusion: '',
    },
  },
  {
    id: 'mammo-masse',
    label: 'Mammographie avec masse',
    technical_id: 'TEST-MAMMO-002',
    patient_id: '11600',
    accession_number: 'ACC2024TEST02',
    study_instance_uid: '5.1.600.598386.1.618.1.905951.1.33160890.148',
    modality: 'MG',
    structured: {
      title: 'Mammographie bilaterale',
      indication: 'Masse palpable sein droit',
      technique: 'Incidences CC et MLO bilaterales + cliches complementaires',
      results: 'Sein droit: masse dense, contours irreguliers, QSE, 15mm. BI-RADS 4.\nSein gauche: BI-RADS 1.',
      conclusion: '',
    },
  },
  {
    id: 'mammo-foyer',
    label: 'Mammographie foyer tissulaire',
    technical_id: 'TEST-MAMMO-003',
    patient_id: '006',
    accession_number: 'ACC2024TEST03',
    study_instance_uid: '2.16.840.1.113669.632.21.1088025335.316606676.10001221047',
    modality: 'MG',
    structured: {
      title: 'Mammographie bilaterale',
      indication: 'Controle foyer tissulaire',
      technique: 'Incidences CC et MLO bilaterales',
      results: 'Sein droit: foyer de densite tissulaire retroareolaire, stable. BI-RADS 3.\nSein gauche: BI-RADS 1.',
      conclusion: '',
    },
  },
];

interface StepLog {
  step: string;
  status: 'pending' | 'running' | 'success' | 'error';
  message?: string;
}

interface RisSimulatorProps {
  isTauriApp: boolean;
}

export const RisSimulator = ({ isTauriApp }: RisSimulatorProps) => {
  const [selectedDataset, setSelectedDataset] = useState(TEST_DATASETS[0].id);
  const [isRunning, setIsRunning] = useState(false);
  const [steps, setSteps] = useState<StepLog[]>([]);

  const updateStep = (index: number, update: Partial<StepLog>) => {
    setSteps(prev => prev.map((s, i) => (i === index ? { ...s, ...update } : s)));
  };

  const getApiKey = useCallback(async (): Promise<string | null> => {
    try {
      const { invoke } = await import('@tauri-apps/api/tauri');
      const keys = await invoke<Array<{ key_value: string }>>('db_list_api_keys');
      const active = keys?.[0];
      return active?.key_value ?? null;
    } catch {
      return null;
    }
  }, []);

  const runSimulation = useCallback(async () => {
    const dataset = TEST_DATASETS.find(d => d.id === selectedDataset);
    if (!dataset) return;

    setIsRunning(true);
    setSteps([
      { step: '1. Récupération clé API', status: 'pending' },
      { step: '2. POST /pending-report', status: 'pending' },
      { step: '3. POST /open-report', status: 'pending' },
    ]);

    // Step 1: Get API key
    updateStep(0, { status: 'running' });
    const apiKey = await getApiKey();
    if (!apiKey) {
      updateStep(0, { status: 'error', message: 'Aucune clé API trouvée en BDD' });
      setIsRunning(false);
      return;
    }
    updateStep(0, { status: 'success', message: `Clé: ${apiKey.substring(0, 16)}...` });

    // Step 2: POST /pending-report
    updateStep(1, { status: 'running' });
    try {
      const body = {
        technical_id: dataset.technical_id,
        patient_id: dataset.patient_id,
        accession_number: dataset.accession_number,
        study_instance_uid: dataset.study_instance_uid,
        modality: dataset.modality,
        source_type: 'teo_hub',
        structured: dataset.structured,
      };
      const res = await fetch('http://localhost:8741/pending-report', {
        method: 'POST',
        headers: { 'X-API-Key': apiKey, 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      });
      const data = await res.json();
      if (res.ok) {
        updateStep(1, { status: 'success', message: `tid=${data.technical_id}` });
        logger.info('RIS Sim: pending-report OK', data);
      } else {
        updateStep(1, { status: 'error', message: data.error || `HTTP ${res.status}` });
        setIsRunning(false);
        return;
      }
    } catch (err) {
      updateStep(1, { status: 'error', message: String(err) });
      setIsRunning(false);
      return;
    }

    // Step 3: POST /open-report
    updateStep(2, { status: 'running' });
    try {
      const res = await fetch(`http://localhost:8741/open-report?tid=${dataset.technical_id}`, {
        method: 'POST',
      });
      const data = await res.json();
      if (res.ok) {
        updateStep(2, { status: 'success', message: data.navigated_to || 'Navigation OK' });
        logger.info('RIS Sim: open-report OK', data);
      } else {
        updateStep(2, { status: 'error', message: data.error || `HTTP ${res.status}` });
      }
    } catch (err) {
      updateStep(2, { status: 'error', message: String(err) });
    }

    setIsRunning(false);
  }, [selectedDataset, getApiKey]);

  if (!isTauriApp) return null;

  return (
    <div className="space-y-3">
      <h4 className="text-sm font-semibold flex items-center gap-2">
        <FlaskConical className="w-4 h-4" />
        Simulation RIS
      </h4>

      <Select value={selectedDataset} onValueChange={setSelectedDataset}>
        <SelectTrigger className="h-8 text-xs">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {TEST_DATASETS.map(d => (
            <SelectItem key={d.id} value={d.id} className="text-xs">
              {d.label} (PID: {d.patient_id})
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      <Button
        variant="outline"
        size="sm"
        className="w-full"
        onClick={runSimulation}
        disabled={isRunning}
      >
        {isRunning ? (
          <Loader2 className="w-3 h-3 mr-2 animate-spin" />
        ) : (
          <Play className="w-3 h-3 mr-2" />
        )}
        Simuler appel RIS complet
      </Button>

      {steps.length > 0 && (
        <Card className="bg-background/50">
          <CardContent className="p-3 space-y-2">
            {steps.map((s, i) => (
              <div key={i} className="flex items-start gap-2 text-xs">
                {s.status === 'pending' && <div className="w-3 h-3 rounded-full bg-muted mt-0.5" />}
                {s.status === 'running' && <Loader2 className="w-3 h-3 animate-spin text-primary mt-0.5" />}
                {s.status === 'success' && <CheckCircle2 className="w-3 h-3 text-primary mt-0.5" />}
                {s.status === 'error' && <XCircle className="w-3 h-3 text-destructive mt-0.5" />}
                <div className="flex-1 min-w-0">
                  <span className="font-medium">{s.step}</span>
                  {s.message && (
                    <p className={`text-[10px] break-all ${s.status === 'error' ? 'text-destructive' : 'text-muted-foreground'}`}>
                      {s.message}
                    </p>
                  )}
                </div>
              </div>
            ))}
          </CardContent>
        </Card>
      )}
    </div>
  );
};
