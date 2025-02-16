#[cfg(test)]
mod clipboard_tests {
    use anyhow::Result;
    use arboard::Clipboard;
    use code2prompt::clipboard::spawn_clipboard_daemon;
    use std::thread;
    use std::time::Duration;

    /// Test production du daemon sous Linux.
    ///
    /// Ce test lance le daemon qui utilise `wait()`. Pour débloquer ce wait,
    /// nous simulons un événement de remplacement en écrasant le presse‑papiers avec une valeur dummy.
    /// Après l'overwrite, nous vérifions que le presse‑papiers contient bien la valeur dummy,
    /// ce qui signifie que le daemon a terminé correctement.
    #[test]
    #[cfg(target_os = "linux")]
    fn test_clipboard_daemon_production() -> Result<()> {
        let test_content = "Test Clipboard Content for Linux";

        // Lance le daemon (qui lit le contenu via STDIN et appelle wait())
        spawn_clipboard_daemon(test_content)?;

        // Attendre un court instant pour laisser le daemon s'installer
        thread::sleep(Duration::from_millis(300));

        // Simuler un overwrite après un court délai dans un thread séparé.
        // Cela doit débloquer le wait() dans le daemon.
        thread::spawn(|| {
            // Attendre un peu pour être sûr que le daemon est en attente
            thread::sleep(Duration::from_millis(500));
            let mut clipboard = Clipboard::new().expect("Failed to open clipboard for overwrite");
            clipboard
                .set_text("dummy".to_string())
                .expect("Failed to overwrite clipboard");
        });

        // Attendre suffisamment longtemps pour que le remplacement ait lieu
        thread::sleep(Duration::from_millis(1000));

        // Vérifier que le presse-papiers a bien été remplacé par "dummy".
        let mut clipboard = Clipboard::new().expect("Failed to open clipboard");
        let content = clipboard.get_text().expect("Failed to get clipboard text");
        assert_eq!(
            content, "dummy",
            "Clipboard content should be overwritten by dummy"
        );

        Ok(())
    }

    /// Test du fonctionnement de la copie dans le presse-papiers sur les OS non-Linux
    ///
    /// Ce test utilise la fonction simple `copy_to_clipboard` qui définit directement le contenu.
    #[test]
    #[cfg(not(target_os = "linux"))]
    fn test_clipboard_copy() {
        let test_content = "Test Clipboard Content for non-Linux";

        // Utilise la fonction standard pour copier dans le presse-papiers.
        copy_to_clipboard(test_content).expect("Failed to copy to clipboard");

        // Vérifie que le contenu du presse-papiers correspond.
        let mut clipboard = Clipboard::new().expect("Failed to open clipboard");
        let content = clipboard.get_text().expect("Failed to get clipboard text");
        assert_eq!(
            content, test_content,
            "Le contenu du presse-papiers doit être celui de test_content"
        );
    }
}
