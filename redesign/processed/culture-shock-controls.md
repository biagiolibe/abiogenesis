# Culture Shock — controlli

Documento autonomo. Sostituisce e unifica lo schema controlli sparso tra GDD §11, `abiogenesis-actions.md`, `culture-shock-inspect-tool.md` e `abiogenesis-hud-notebook.md` — documenti scritti in momenti diversi che, messi vicini, si contraddicono su più punti. Introduce anche un menu di pausa, mai definito finora.

## I conflitti che questo documento risolve

- **Zoom senza controllo assegnato.** La mappa a due livelli (overview/dettaglio, `culture-shock-population-model-aesthetic.md`) non ha mai avuto un modo per passare dall'uno all'altro.
- **Click sovraccaricato.** Il click sinistro doveva sia eseguire un'azione armata sia aprire la card di ispezione — nessun documento diceva come distinguere i due casi.
- **Nessun modo di disarmare un'azione.** Selezionata un'azione dall'HUD, non esisteva un modo per tornare alla sola osservazione senza eseguirla.
- **`Esc` con tre usi mai riconciliati.** Uscita dal gioco (riepilogo controlli), chiusura della card di ispezione, e — nel documento notebook — non veniva nemmeno usato per chiudere il notebook (solo Tab o click fuori). Il rischio concreto: premere Esc per errore mentre si chiude una card avrebbe fatto uscire dal gioco, pericoloso con uno slot di salvataggio unico consumato al caricamento.
- **`Space` ancorato all'era mentre la stagione è l'unità di decisione.** La revisione della scala temporale ha reso la stagione il ritmo principale del giocatore, ma il tasto più comodo continuava ad avanzare l'unità più rara e pesante.
- **Avanzamento continuo e "vai al prossimo evento notevole"**, entrambi proposti altrove, mai assegnati a un tasto.
- **`R` (reseed) senza protezione**, nonostante ora esista un salvataggio da poter perdere per errore.

## Schema controlli — mouse

| Input | Azione |
|---|---|
| Passaggio del mouse | Tooltip leggero, sempre attivo (nome, popolazione, trend) |
| Click sinistro, nessuna azione armata | Apre/aggancia la card di ispezione completa sulla cella (comportamento di default) |
| Click sinistro, azione armata | Esegue l'azione armata sulla cella |
| Click destro | **Disarma** l'azione corrente, torna a modalità osservazione — senza eseguire nulla |
| Rotellina | Zoom tra overview e dettaglio |
| Trascinamento (tasto centrale o mano) | Pan della camera |

## Schema controlli — tastiera

| Tasto | Azione |
|---|---|
| `1`–`4` | Selezione rapida di Seed / Stress / Cull / Splice (equivalente da tastiera del click sull'icona HUD) |
| `Space` | Avanza una **stagione** |
| `Shift+Space` | Avanza un'intera **era** |
| `N` | Avanza un singolo pulse (osservazione fine/debug) |
| `P` | Attiva/disattiva l'avanzamento continuo |
| `G` | Avanza automaticamente fino al prossimo evento sopra soglia di rilevanza |
| `Tab` | Apre/chiude il notebook |
| `Esc` | Chiude il livello di interfaccia più in alto attualmente aperto (vedi sotto) |
| `R` | Reseed del mondo corrente — **protetto**, vedi sotto |
| `WASD` / frecce | Pan della camera |

### `Space` ora avanza la stagione, non l'era

Correzione diretta del disallineamento: con la stagione come unità di decisione (`abiogenesis-time-scale-reveal.md`), il tasto più comodo deve corrispondere all'azione più frequente. Avanzare un'era intera resta disponibile (`Shift+Space`), ma non è più il default.

### `Esc` a strati, mai un'uscita diretta

Comportamento a cascata, dal livello più in alto verso il basso:

1. Se il notebook è aperto → lo chiude.
2. Altrimenti, se una card di ispezione è agganciata → la chiude.
3. Altrimenti, se un'azione è armata → la disarma (stesso effetto del click destro).
4. Altrimenti → apre il **menu di pausa** (nuovo, sotto) — mai un'uscita immediata dal gioco.

Nessun singolo tocco di Esc esce mai dal gioco direttamente. Necessario soprattutto per il rischio di salvataggio: con uno slot unico consumato al caricamento (`abiogenesis-cross-cutting.md`), un'uscita accidentale ha un costo reale.

### `R` protetto

Il reseed ha senso solo su un mondo non ancora toccato dal giocatore. Se sono già state eseguite azioni in quel mondo, `R` richiede una conferma esplicita invece di eseguire immediatamente — stessa logica già applicata altrove (uscita da un mondo, cfr. `abiogenesis-transitions-metaprogression.md`): mai un'azione irreversibile senza che il giocatore l'abbia confermata sapendo cosa comporta.

## Menu di pausa — nuovo

Non era mai stato definito: le impostazioni erano raggiungibili "dal menu principale o da una pausa in-run" (`abiogenesis-cross-cutting.md`), ma la pausa stessa non esisteva come schermata. Contenuto minimo:

- **Riprendi** (chiude il menu, torna al gioco).
- **Impostazioni** (stesso pannello raggiungibile dal menu principale).
- **Salva ed esci** — unico modo sicuro di uscire durante una run, esplicito e consapevole.
- **Abbandona senza salvare** — disponibile ma visivamente distinto (es. colore d'allerta), per chi vuole davvero interrompere senza consumare lo slot.

Il tempo è in pausa mentre questo menu è aperto — stessa regola già valida per il notebook.

## Cosa serve per l'integrazione

- **Stato di "azione armata"** esplicito e visibile nell'HUD (l'icona selezionata già lo mostra, cfr. `abiogenesis-hud-notebook.md` — va solo confermato che disarmare la resetti visivamente).
- **Gestore di input a priorità**, che implementi la cascata di `Esc` sopra — un solo listener che controlla lo stato di notebook/card/azione armata in quest'ordine, non tre gestori indipendenti.
- **Flag "mondo toccato"** per la protezione di `R` — vero dal primo `Seed`/`Stress`/`Cull`/`Splice` eseguito in quel mondo.
- **Schermata menu di pausa**, riusando il layout del menu principale dove possibile (stesso stile console/ardesia già stabilito).
- **Aggiornare il riepilogo "come si gioca"** (`abiogenesis-menu-onboarding.md`) con lo schema controlli qui definito, che sostituisce quello precedente.

## Fuori scope

- Rebinding dei tasti — nessuna personalizzazione prevista in questa fase.
- Supporto controller/gamepad — non valutato, coerente con la scelta "solo PC, input mouse+tastiera" di `culture-shock-distribution.md`.
- Layout esatto del menu di pausa — qui solo il contenuto minimo, non una specifica pixel-perfect.
