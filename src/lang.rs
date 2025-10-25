use super::Language;
use std::collections::HashMap;

pub fn get_translations(lang: Language) -> HashMap<&'static str, &'static str> {
    match lang {
        Language::English => HashMap::from([
            // Registration
            ("welcome_title", "Welcome!"),
            ("create_acc_prompt", "Create your account to start managing your passwords securely."),
            ("registration", "📝 Registration"),
            ("username", "👤 Username:"),
            ("password", "🔑 Password:"),
            ("confirm", "🔑 Confirm:"),
            ("more_char", "💡 Your password requires at least 6 characters"),
            ("register", "Register"),
            
            // Login
            ("welcome", "Welcome back!"),
            ("login_prompt", "Enter your credentials to access."),
            ("login_title", "🔓 Login"),
            ("login_button", "Login"),
            
            // Main UI
            ("password_manager", "🔐 Password Manager"),
            ("exit", "🚪 Exit"),
            ("add_tab", "➕ Add"),
            ("edit_tab", "⚙ Edit"),
            
            // Add Password Panel
            ("add_password_title", "➕ Add Password"),
            ("service_name", "🏷 Service name"),
            ("service_hint", "e.g. Gmail, Facebook..."),
            ("username_label", "👤 Username"),
            ("username_hint", "username or email"),
            ("password_label", "🔑 Password"),
            ("password_hint", "secure password"),
            ("save_password", "💾 Save Password"),
            
            // Edit Password Panel
            ("edit_password_title", "⚙ Edit Password"),
            ("service_to_edit", "🎯 Service to edit"),
            ("service_exists_hint", "Existing service name"),
            ("new_username", "👤 New username (optional)"),
            ("leave_empty", "Leave empty to keep unchanged"),
            ("new_password", "🔑 New password"),
            ("new_password_hint", "New secure password"),
            ("confirm_password", "🔑 Confirm password"),
            ("repeat_password", "Repeat the new password"),
            ("edit_button", "🔄 Edit Password"),
            
            // Password List
            ("your_passwords", "📃 Your Passwords"),
            ("search_placeholder", "🔍 Search..."),
            ("clear_search", "Clear search"),
            ("results_found", "🎯 {} results found"),
            ("no_passwords", "📭 No passwords saved"),
            ("add_first_password", "Add your first password using the left panel"),
            ("no_results", "🔍 No results"),
            ("no_password_for", "No password found for '{}'"),
            ("password_protected", "🔒 Password protected"),
            ("decryption_error", "⚠ Decryption error"),
            ("key_unavailable", "⚠ Key unavailable"),
            
            // Buttons and tooltips
            ("delete", "Delete"),
            ("show_password", "Show Password"),
            ("copy_password", "Copy password"),
            ("copy_username", "Copy username"),
            ("change_theme", "Change theme"),
            
            // Messages
            ("password_copied", "Password for '{}' has been copied!"),
            ("username_copied", "Username for '{}' has been copied!"),
            ("password_deleted", "Password for '{}' has been deleted!"),
            ("passwords_deleted", "{} passwords deleted!"),
            ("decryption_failed", "Decryption error!"),
            ("key_not_available", "Encryption key not available!"),
            ("password_added", "Password added successfully!"),
            ("password_updated", "Password updated successfully!"),
            ("service_not_found", "Service '{}' not found!"),
            ("passwords_dont_match", "Passwords don't match!"),
            ("fill_all_fields", "Please fill in all fields!"),
            ("password_too_short", "Password must be at least 6 characters!"),
            ("username_exists", "Username already exists!"),
            ("invalid_credentials", "Invalid username or password!"),
            ("registration_success", "Registration successful! Please login."),
        ]),
        
        Language::Italian => HashMap::from([
            // Registrazione
            ("welcome_title", "Benvenuto!"),
            ("create_acc_prompt", "Crea il tuo account per iniziare a gestire le tue password in sicurezza."),
            ("registration", "📝 Registrazione"),
            ("username", "👤 Username:"),
            ("password", "🔑 Password:"),
            ("confirm", "🔑 Conferma:"),
            ("more_char", "💡 La password deve essere di almeno 6 caratteri"),
            ("register", "Registrati"),
            
            // Login
            ("welcome", "Bentornato!"),
            ("login_prompt", "Inserisci le tue credenziali per accedere."),
            ("login_title", "🔓 Accesso"),
            ("login_button", "Accedi"),
            
            // UI Principale
            ("password_manager", "🔐 Password Manager"),
            ("exit", "🚪 Esci"),
            ("add_tab", "➕ Aggiungi"),
            ("edit_tab", "⚙ Modifica"),
            
            // Pannello Aggiungi Password
            ("add_password_title", "➕ Aggiungi Password"),
            ("service_name", "🏷 Nome servizio"),
            ("service_hint", "es. Gmail, Facebook..."),
            ("username_label", "👤 Username"),
            ("username_hint", "username o email"),
            ("password_label", "🔑 Password"),
            ("password_hint", "password sicura"),
            ("save_password", "💾 Salva Password"),
            
            // Pannello Modifica Password
            ("edit_password_title", "⚙ Modifica Password"),
            ("service_to_edit", "🎯 Servizio da modificare"),
            ("service_exists_hint", "Nome del servizio esistente"),
            ("new_username", "👤 Nuovo username (opzionale)"),
            ("leave_empty", "Lascia vuoto per non modificare"),
            ("new_password", "🔑 Nuova password"),
            ("new_password_hint", "Nuova password sicura"),
            ("confirm_password", "🔑 Conferma password"),
            ("repeat_password", "Ripeti la nuova password"),
            ("edit_button", "🔄 Modifica Password"),
            
            // Lista Password
            ("your_passwords", "📃 Le tue Password"),
            ("search_placeholder", "🔍 Cerca..."),
            ("clear_search", "Cancella ricerca"),
            ("results_found", "🎯 {} risultati trovati"),
            ("no_passwords", "📭 Nessuna password salvata"),
            ("add_first_password", "Aggiungi la tua prima password usando il pannello a sinistra"),
            ("no_results", "🔍 Nessun risultato"),
            ("no_password_for", "Nessuna password trovata per '{}'"),
            ("password_protected", "🔒 Password protetta"),
            ("decryption_error", "⚠ Errore decrittografia"),
            ("key_unavailable", "⚠ Chiave non disponibile"),
            
            // Pulsanti e tooltip
            ("delete", "Elimina"),
            ("show_password", "Mostra Password"),
            ("copy_password", "Copia password"),
            ("copy_username", "Copia username"),
            ("change_theme", "Cambia tema"),
            
            // Messaggi
            ("password_copied", "La password di '{}' è stata copiata!"),
            ("username_copied", "L'username di '{}' è stato copiato!"),
            ("password_deleted", "La password di '{}' è stata eliminata!"),
            ("passwords_deleted", "{} password eliminate!"),
            ("decryption_failed", "Errore nella decrittografia!"),
            ("key_not_available", "Chiave di crittografia non disponibile!"),
            ("password_added", "Password aggiunta con successo!"),
            ("password_updated", "Password aggiornata con successo!"),
            ("service_not_found", "Servizio '{}' non trovato!"),
            ("passwords_dont_match", "Le password non corrispondono!"),
            ("fill_all_fields", "Compila tutti i campi!"),
            ("password_too_short", "La password deve essere di almeno 6 caratteri!"),
            ("username_exists", "Username già esistente!"),
            ("invalid_credentials", "Username o password non validi!"),
            ("registration_success", "Registrazione completata! Effettua il login."),
        ]),
    }
}