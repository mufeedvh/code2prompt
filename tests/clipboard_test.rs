// #[cfg(test)]
// mod clipboard_tests {
//     use arboard::Clipboard;
//     use code2prompt::copy_text_to_clipboard;

//     /// Test du fonctionnement de la copie dans le presse-papiers sur les OS non-Linux
//     ///
//     /// Ce test utilise la fonction simple `copy_to_clipboard` qui définit directement le contenu.
//     #[test]
//     fn test_clipboard_copy() {
//         let test_content = "Test Clipboard Content for non-Linux";

//         // Utilise la fonction standard pour copier dans le presse-papiers.
//         copy_text_to_clipboard(test_content).expect("Failed to copy to clipboard");

//         // Vérifie que le contenu du presse-papiers correspond.
//         let mut clipboard = Clipboard::new().expect("Failed to open clipboard");
//         let content = clipboard.get_text().expect("Failed to get clipboard text");
//         assert_eq!(
//             content, test_content,
//             "Le contenu du presse-papiers doit être celui de test_content"
//         );
//     }
// }
