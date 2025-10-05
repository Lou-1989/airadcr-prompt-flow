fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icons/icon.ico")
            .set("ProductName", "AIRADCR - Dictée Radiologique Professionnelle")
            .set("CompanyName", "AIRADCR (https://airadcr.com)")
            .set("FileDescription", "Application médicale professionnelle de dictée radiologique développée par AIRADCR pour les radiologues. Site officiel : https://airadcr.com")
            .set("LegalCopyright", "Copyright © 2025 AIRADCR. Tous droits réservés. https://airadcr.com - Développé par Dr Lounes BENSID")
            .set("InternalName", "AIRADCR")
            .set("OriginalFilename", "AIRADCR.exe")
            .set("ProductVersion", "1.0.0")
            .set("FileVersion", "1.0.0");
        
        if let Err(e) = res.compile() {
            eprintln!("Error compiling Windows resources: {}", e);
            std::process::exit(1);
        }
    }
    
    #[cfg(not(windows))]
    {
        // Rien à faire sur les autres plateformes
    }
    
    tauri_build::build()
}
