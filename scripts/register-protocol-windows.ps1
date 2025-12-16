# ============================================================================
# AIRADCR Desktop - Enregistrement du protocole airadcr:// (Windows)
# ============================================================================
# Ce script enregistre le protocole URL airadcr:// dans le registre Windows
# pour permettre le lancement de l'application depuis le navigateur ou le RIS.
#
# Usage:
#   - Exécuter en tant qu'administrateur pour enregistrement système
#   - Ou sans admin pour enregistrement utilisateur uniquement
#
# Formats supportés après installation:
#   - airadcr://open?tid=ABC123
#   - airadcr://open/ABC123
#   - airadcr://ABC123
# ============================================================================

param(
    [string]$ExePath = "",
    [switch]$Uninstall = $false,
    [switch]$UserOnly = $false
)

# Détection automatique du chemin de l'exe
if ([string]::IsNullOrEmpty($ExePath)) {
    # Chemins possibles
    $possiblePaths = @(
        "$env:ProgramFiles\AIRADCR\AIRADCR.exe",
        "$env:LOCALAPPDATA\AIRADCR\AIRADCR.exe",
        "$PSScriptRoot\..\src-tauri\target\release\airadcr-desktop.exe",
        "$PSScriptRoot\..\src-tauri\target\debug\airadcr-desktop.exe"
    )
    
    foreach ($path in $possiblePaths) {
        if (Test-Path $path) {
            $ExePath = $path
            break
        }
    }
}

if ([string]::IsNullOrEmpty($ExePath) -and -not $Uninstall) {
    Write-Host "❌ Erreur: Chemin de l'exécutable AIRADCR non trouvé." -ForegroundColor Red
    Write-Host "   Spécifiez le chemin avec: -ExePath 'C:\chemin\vers\AIRADCR.exe'" -ForegroundColor Yellow
    exit 1
}

# Déterminer la clé de registre (HKLM pour système, HKCU pour utilisateur)
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if ($UserOnly -or -not $isAdmin) {
    $registryBase = "HKCU:\Software\Classes"
    $scope = "utilisateur"
} else {
    $registryBase = "HKLM:\Software\Classes"
    $scope = "système"
}

$protocolKey = "$registryBase\airadcr"

# Mode désinstallation
if ($Uninstall) {
    Write-Host "🗑️  Désinstallation du protocole airadcr:// ($scope)..." -ForegroundColor Cyan
    
    if (Test-Path $protocolKey) {
        Remove-Item -Path $protocolKey -Recurse -Force
        Write-Host "✅ Protocole airadcr:// supprimé avec succès" -ForegroundColor Green
    } else {
        Write-Host "⚠️  Le protocole airadcr:// n'était pas enregistré" -ForegroundColor Yellow
    }
    exit 0
}

# Mode installation
Write-Host "🔗 Enregistrement du protocole airadcr:// ($scope)..." -ForegroundColor Cyan
Write-Host "   Exécutable: $ExePath" -ForegroundColor Gray

# Vérifier que l'exe existe
if (-not (Test-Path $ExePath)) {
    Write-Host "❌ Erreur: L'exécutable n'existe pas: $ExePath" -ForegroundColor Red
    exit 1
}

# Créer les clés de registre
try {
    # Clé principale du protocole
    if (-not (Test-Path $protocolKey)) {
        New-Item -Path $protocolKey -Force | Out-Null
    }
    
    # Propriétés du protocole
    Set-ItemProperty -Path $protocolKey -Name "(Default)" -Value "URL:AIRADCR Protocol"
    Set-ItemProperty -Path $protocolKey -Name "URL Protocol" -Value ""
    
    # Icône par défaut
    $iconKey = "$protocolKey\DefaultIcon"
    if (-not (Test-Path $iconKey)) {
        New-Item -Path $iconKey -Force | Out-Null
    }
    Set-ItemProperty -Path $iconKey -Name "(Default)" -Value "`"$ExePath`",0"
    
    # Commande d'ouverture
    $commandKey = "$protocolKey\shell\open\command"
    if (-not (Test-Path $commandKey)) {
        New-Item -Path $commandKey -Force | Out-Null
    }
    Set-ItemProperty -Path $commandKey -Name "(Default)" -Value "`"$ExePath`" `"%1`""
    
    Write-Host ""
    Write-Host "✅ Protocole airadcr:// enregistré avec succès!" -ForegroundColor Green
    Write-Host ""
    Write-Host "📋 Formats supportés:" -ForegroundColor Cyan
    Write-Host "   • airadcr://open?tid=ABC123" -ForegroundColor White
    Write-Host "   • airadcr://open/ABC123" -ForegroundColor White
    Write-Host "   • airadcr://ABC123" -ForegroundColor White
    Write-Host ""
    Write-Host "🧪 Test: Ouvrez ce lien dans votre navigateur:" -ForegroundColor Cyan
    Write-Host "   airadcr://open?tid=TEST123" -ForegroundColor Yellow
    Write-Host ""
    
} catch {
    Write-Host "❌ Erreur lors de l'enregistrement: $_" -ForegroundColor Red
    exit 1
}
