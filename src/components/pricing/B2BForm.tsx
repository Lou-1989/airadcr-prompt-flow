import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Building2 } from "lucide-react";

interface BusinessInfo {
  companyName: string;
  siret: string;
  vatNumber: string;
}

interface B2BFormProps {
  businessInfo: BusinessInfo;
  onChange: (info: BusinessInfo) => void;
  errors?: Partial<Record<keyof BusinessInfo, string>>;
}

export function B2BForm({ businessInfo, onChange, errors }: B2BFormProps) {
  const handleChange = (field: keyof BusinessInfo) => (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange({ ...businessInfo, [field]: e.target.value });
  };

  return (
    <Card className="border-primary/20 bg-primary/5">
      <CardHeader className="pb-4">
        <CardTitle className="flex items-center gap-2 text-lg">
          <Building2 className="h-5 w-5" />
          Informations entreprise
        </CardTitle>
        <CardDescription>
          Pour la facturation professionnelle avec TVA
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="companyName">Raison sociale *</Label>
          <Input
            id="companyName"
            placeholder="Nom de l'entreprise"
            value={businessInfo.companyName}
            onChange={handleChange("companyName")}
            maxLength={100}
            className={errors?.companyName ? "border-destructive" : ""}
          />
          {errors?.companyName && (
            <p className="text-sm text-destructive">{errors.companyName}</p>
          )}
        </div>

        <div className="space-y-2">
          <Label htmlFor="siret">SIRET (optionnel)</Label>
          <Input
            id="siret"
            placeholder="123 456 789 00012"
            value={businessInfo.siret}
            onChange={handleChange("siret")}
            maxLength={17}
          />
          <p className="text-xs text-muted-foreground">
            14 chiffres pour les entreprises françaises
          </p>
        </div>

        <div className="space-y-2">
          <Label htmlFor="vatNumber">N° TVA intracommunautaire (optionnel)</Label>
          <Input
            id="vatNumber"
            placeholder="FR12345678901 ou CHE-123.456.789"
            value={businessInfo.vatNumber}
            onChange={handleChange("vatNumber")}
            maxLength={20}
          />
          <p className="text-xs text-muted-foreground">
            Requis pour l'exonération de TVA intracommunautaire
          </p>
        </div>
      </CardContent>
    </Card>
  );
}
