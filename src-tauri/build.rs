fn main() {
    // Tauri gère automatiquement les métadonnées Windows depuis Cargo.toml
    // Pas besoin de winres qui crée un conflit de ressources
    tauri_build::build()
}
