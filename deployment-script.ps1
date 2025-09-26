# Script PowerShell - Installation AirADCR Desktop pour Windows
# À exécuter en tant qu'administrateur pour installation système

param(
    [string]$AppUrl = "https://votre-domaine-airadcr.lovable.app",
    [switch]$AllUsers = $false,
    [switch]$StartupShortcut = $false
)

Write-Host "🔧 Installation AirADCR Desktop pour Windows..." -ForegroundColor Cyan

# Configuration
$AppName = "AirADCR Desktop"
$ShortcutName = "AirADCR Desktop.lnk"
$ChromePath = "${env:ProgramFiles}\Google\Chrome\Application\chrome.exe"
$EdgePath = "${env:ProgramFiles(x86)}\Microsoft\Edge\Application\msedge.exe"

# Déterminer le navigateur disponible
$BrowserPath = $null
$BrowserArgs = $null

if (Test-Path $ChromePath) {
    $BrowserPath = $ChromePath
    $BrowserArgs = "--app=`"$AppUrl`" --window-size=1200,800 --disable-web-security --user-data-dir=`"$env:APPDATA\AirADCR-Chrome`""
    Write-Host "✅ Chrome détecté" -ForegroundColor Green
} elseif (Test-Path $EdgePath) {
    $BrowserPath = $EdgePath
    $BrowserArgs = "--app=`"$AppUrl`" --window-size=1200,800 --disable-web-security --user-data-dir=`"$env:APPDATA\AirADCR-Edge`""
    Write-Host "✅ Edge détecté" -ForegroundColor Green
} else {
    Write-Host "❌ Aucun navigateur compatible trouvé (Chrome ou Edge requis)" -ForegroundColor Red
    exit 1
}

# Déterminer les dossiers de destination
if ($AllUsers) {
    $DesktopPath = "$env:PUBLIC\Desktop"
    $StartupPath = "$env:ALLUSERSPROFILE\Microsoft\Windows\Start Menu\Programs\Startup"
} else {
    $DesktopPath = [Environment]::GetFolderPath("Desktop")
    $StartupPath = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup"
}

# Créer le raccourci bureau
try {
    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut("$DesktopPath\$ShortcutName")
    $Shortcut.TargetPath = $BrowserPath
    $Shortcut.Arguments = $BrowserArgs
    $Shortcut.Description = "Application desktop AirADCR avec injection système"
    $Shortcut.IconLocation = "$BrowserPath,0"  # Icône du navigateur par défaut
    $Shortcut.WindowStyle = 1  # Fenêtre normale
    $Shortcut.Save()
    
    Write-Host "✅ Raccourci bureau créé : $DesktopPath\$ShortcutName" -ForegroundColor Green
} catch {
    Write-Host "❌ Erreur création raccourci bureau : $($_.Exception.Message)" -ForegroundColor Red
}

# Créer le raccourci démarrage automatique (optionnel)
if ($StartupShortcut) {
    try {
        $StartupShortcut = $WshShell.CreateShortcut("$StartupPath\$ShortcutName")
        $StartupShortcut.TargetPath = $BrowserPath
        $StartupShortcut.Arguments = $BrowserArgs
        $StartupShortcut.Description = "Démarrage automatique AirADCR Desktop"
        $StartupShortcut.IconLocation = "$BrowserPath,0"
        $StartupShortcut.WindowStyle = 7  # Fenêtre minimisée
        $StartupShortcut.Save()
        
        Write-Host "✅ Démarrage automatique configuré" -ForegroundColor Green
    } catch {
        Write-Host "⚠️ Impossible de configurer le démarrage automatique : $($_.Exception.Message)" -ForegroundColor Yellow
    }
}

# Créer un dossier de données utilisateur
$UserDataPath = "$env:APPDATA\AirADCR"
if (-not (Test-Path $UserDataPath)) {
    New-Item -Path $UserDataPath -ItemType Directory -Force | Out-Null
    Write-Host "✅ Dossier utilisateur créé : $UserDataPath" -ForegroundColor Green
}

# Créer un fichier de configuration
$ConfigContent = @"
[AirADCR Desktop Configuration]
AppUrl=$AppUrl
Version=1.0.0
InstallDate=$(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
BrowserPath=$BrowserPath
AutoStartup=$StartupShortcut
InstallationType=$(if ($AllUsers) { "System" } else { "User" })
"@

$ConfigPath = "$UserDataPath\config.ini"
$ConfigContent | Out-File -FilePath $ConfigPath -Encoding UTF8
Write-Host "✅ Configuration sauvegardée : $ConfigPath" -ForegroundColor Green

# Message de fin
Write-Host ""
Write-Host "🎉 Installation AirADCR Desktop terminée !" -ForegroundColor Green
Write-Host ""
Write-Host "📋 Résumé :" -ForegroundColor Cyan
Write-Host "   • Application URL : $AppUrl"
Write-Host "   • Navigateur utilisé : $(Split-Path $BrowserPath -Leaf)"
Write-Host "   • Raccourci bureau : $DesktopPath\$ShortcutName"
if ($StartupShortcut) {
    Write-Host "   • Démarrage automatique : Activé"
}
Write-Host ""
Write-Host "🚀 Pour lancer AirADCR Desktop :" -ForegroundColor Yellow
Write-Host "   • Double-cliquer sur le raccourci bureau"
Write-Host "   • Ou rechercher 'AirADCR' dans le menu Démarrer"
Write-Host ""
Write-Host "🛠️ Support : Configuration dans $ConfigPath" -ForegroundColor Gray

# Optionnel : Lancer l'application immédiatement
$LaunchNow = Read-Host "Lancer AirADCR Desktop maintenant ? (O/N)"
if ($LaunchNow -eq "O" -or $LaunchNow -eq "o") {
    Write-Host "🚀 Lancement d'AirADCR Desktop..." -ForegroundColor Cyan
    Start-Process -FilePath $BrowserPath -ArgumentList $BrowserArgs
}