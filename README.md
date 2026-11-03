# InstallDesk

Programma Windows (Rust, egui) per tenere una **cartella di installer sempre aggiornata**
e creare gli **avvii silenziosi** per reinstallare tutto in un colpo.

- **Cartella predefinita**: `Installer` accanto al programma (cambiabile). Ogni programma ha la sua sottocartella.
- **Controlla aggiornamenti**: confronta la versione dei tuoi file con l'ultima di *winget*.
- **Scarica aggiornamenti / Aggiungi programmi**: scarica con `winget download`, verifica che il file sia un installer valido, sposta le versioni precedenti in `Vecchie\` (nulla viene cancellato).
- **Crea script silenziosi**: un `Installa <Programma>.cmd` accanto a ogni installer e `Installa-tutto.cmd` nella cartella principale. Gli script si auto-elevano e scrivono `installazioni.log`.
- Tipo di installer riconosciuto dalla firma (Inno, NSIS, InstallShield, WiX Burn, MSI, MSIX); le opzioni silenziose si possono modificare per ogni programma e vengono ricordate in `installdesk.json`.

Richiede winget (incluso in Windows 10/11). Compilazione: `costruisci-installdesk.cmd`.

## Uso rapido: tre pulsanti

1. **Scansiona il PC**: elenca tutti i programmi installati nel sistema che winget sa scaricare (scheda «Programmi del PC»), con versione installata e disponibile.
2. **Scarica**: scarica gli installer dei programmi selezionati nella cartella predefinita, uno per cartella. Quelli già presenti in versione aggiornata vengono saltati; «Interrompi» ferma la coda. Alla fine crea gli script silenziosi.
3. **Installa**: installa in silenzio gli installer selezionati nella scheda «Installer scaricati» (dopo conferma; Windows chiede l'autorizzazione di amministratore una sola volta).

I programmi non presenti nell'archivio winget non si possono scaricare in automatico: copiali a mano nella cartella.

## Scheda «Estensioni Chrome»

- **Estrai da Chrome**: legge le estensioni di tutti i profili (nome, versione, profilo). Quelle non del Web Store e i temi non sono selezionati di default.
- **Salva elenco**: crea `Estensioni.extdesk.json` più una pagina `.html` con i collegamenti al Web Store.
- **Carica elenco…** + **Installa estensioni** (sul nuovo PC): due modi. *Web Store*: apre le pagine a gruppi di 8 e tu premi «Aggiungi a Chrome» (Chrome non permette l'installazione nascosta). *Installazione forzata*: criteri di Chrome via registro (amministratore); le estensioni risultano «gestite dall'organizzazione» finché non importi `Rimuovi-installazione-forzata.reg`. I file stanno nella cartella `Estensioni` accanto al programma.

## Scheda «Configurazione Windows»

Raccoglie gli script di configurazione (incorporati nel programma): MegaConfig, Italiano + orario, Ottimizza, Attivazione, Blocca Windows Update, Sposta cartelle su D:, Esporta/Importa menu Start e taskbar, Gallery in Esplora file. Ogni pulsante scrive lo script nella cartella `Script` accanto a InstallDesk e lo lancia in una finestra a parte, con l'autorizzazione di amministratore (una sola richiesta se avvii InstallDesk come amministratore). Gli script ad alto impatto (arancioni) chiedono conferma.

`W11ClassicMenu.exe` è di terzi e non è incluso: copialo nella cartella `Script` e il pulsante lo avvia.
