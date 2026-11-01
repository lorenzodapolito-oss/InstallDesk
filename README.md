# InstallDesk

Programma Windows (Rust, egui) per tenere una **cartella di installer sempre aggiornata**
e creare gli **avvii silenziosi** per reinstallare tutto in un colpo.

- **Cartella predefinita**: `Installer` accanto al programma (cambiabile). Ogni programma ha la sua sottocartella.
- **Controlla aggiornamenti**: confronta la versione dei tuoi file con l'ultima di *winget*.
- **Scarica aggiornamenti / Aggiungi programmi**: scarica con `winget download`, verifica che il file sia un installer valido, sposta le versioni precedenti in `Vecchie\` (nulla viene cancellato).
- **Crea script silenziosi**: un `Installa <Programma>.cmd` accanto a ogni installer e `Installa-tutto.cmd` nella cartella principale. Gli script si auto-elevano e scrivono `installazioni.log`.
- Tipo di installer riconosciuto dalla firma (Inno, NSIS, InstallShield, WiX Burn, MSI, MSIX); le opzioni silenziose si possono modificare per ogni programma e vengono ricordate in `installdesk.json`.

Richiede winget (incluso in Windows 10/11). Compilazione: `costruisci-installdesk.cmd`.
