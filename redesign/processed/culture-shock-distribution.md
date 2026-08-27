# Culture Shock — distribuzione

Documento autonomo. Copre piattaforma, modello di rilascio, marketing e prezzo — la prima volta che questi temi vengono affrontati nella serie di documenti di design. Le decisioni qui si appoggiano deliberatamente su cose già decise altrove (roadmap `[PROPOSED]`/post-MVP, determinismo via seed, identità del gioco) invece di introdurre scelte scollegate.

## Piattaforma: PC soltanto

**Decisione:** solo desktop (Windows/Linux/macOS via lo stesso binario Rust), nessun mobile, nessuna console per ora.

**Motivazione:** i controlli già specificati (§11 GDD — click su cella, `space`/`n`/`tab`/`r`/`esc`) sono nativamente desktop. Mobile richiederebbe un ridisegno completo dell'input, non un porting — non ha senso tenerlo come opzione "per dopo" senza una ragione concreta che lo giustifichi. Console aggiunge costo di certificazione senza un pubblico consolidato che lo richieda, per un team piccolo su un titolo di nicchia.

## Canali: itch.io prima, Steam Early Access come casa definitiva

### itch.io — costruire in pubblico

Prima destinazione, non l'unica. Build frequenti, feedback diretto, comunità già abituata a simulazioni sperimentali senza budget artistico — il pubblico più tollerante possibile per un gioco ancora `[PROPOSED]` in metà dei suoi sistemi. Costo di ingresso quasi nullo.

### Steam — Early Access, non lancio pieno

**Osservazione che guida questa decisione:** il roadmap `[PROPOSED]`/post-MVP già scritto in questi documenti **è di fatto già una roadmap di Early Access** — biomi dinamici, eventi catastrofici (`abiogenesis-world-events.md`, `abiogenesis-world-events-catastrophes.md`), il meccanismo di ipotesi dichiarate e smentite (`culture-shock-identity.md`), la meta-progressione concreta (`abiogenesis-transitions-metaprogression.md`, Parte 2). Non serve costruire un piano di comunicazione a parte: serve **usare la classificazione a tre livelli già esistente** (`abiogenesis-system-hierarchy.md` — core / variazione strutturale / payoff raro) come base della roadmap pubblica.

Il pubblico di riferimento (RimWorld, Dwarf Fortress) è abituato a comprare presto e aspettare, e premia proprio la trasparenza su cosa manca e perché — è lo stesso registro onesto ("il gioco dice 'non è chiaro' invece di inventare una causa") che il gioco ha già verso il giocatore, esteso alla comunicazione verso il pubblico.

## Marketing: il seed condivisibile come loop sociale gratuito

**L'osservazione centrale di questo documento.** Il gioco ha già, per altri motivi, tutto ciò che serve per un loop di condivisione spontanea, senza budget di marketing:

- **Determinismo totale via seed** (§5.7 GDD) — un mondo è interamente riproducibile da chiunque abbia lo stesso seed.
- **Il bilancio di fine mondo/run** (`abiogenesis-transitions-metaprogression.md`) — già contiene relazioni confermate, eventi notevoli, la frase di chiusura generata.
- **La Cronaca** (`abiogenesis-notebook-cronaca.md`) — la storia narrata di un mondo, già in forma leggibile.

Insieme, questi tre sistemi bastano a produrre esattamente il tipo di contenuto che ha reso virale Dwarf Fortress per anni: *"guarda cosa è successo nel mio mondo"*, condiviso con un seed replicabile da chiunque. È lo stesso meccanismo, ottenuto gratis da sistemi già progettati per altri motivi.

**Azione concreta:** progettare presto — non aggiungerla a fine sviluppo — una funzione di **esportazione/condivisione del riassunto di un mondo** (testo o immagine, seed incluso), derivata dai dati che il bilancio di fine mondo già calcola. Non è un sistema nuovo da costruire, è un'esposizione in più di dati esistenti — stesso principio già applicato più volte in questi documenti (pipeline del tick, strumento di ispezione).

**Canali coerenti:** comunità Rust/gamedev (il linguaggio è di per sé un gancio in quella nicchia), community hard-SF e roguelike, un devlog regolare che segue naturalmente il ritmo della roadmap Early Access sopra.

## Prezzo

**Fascia bassa dell'indie a pagamento**, indicativamente **10-15$** in Early Access, con margine per salire all'uscita 1.0. Coerente con lo scope attuale — profondo ma non enorme quanto un RimWorld — e con l'assenza di qualunque budget artistico da ammortizzare.

**Free-to-play scartato esplicitamente:** nessun gancio di monetizzazione è mai stato progettato — anzi, la meta-progressione è esplicitamente vincolata a "sblocchi capacità, non risposte" (§10 GDD), incompatibile per principio con acquisti in-app che accelererebbero la scoperta. Introdurre F2P ora contraddirebbe l'identità del gioco appena consolidata (`culture-shock-identity.md`), non solo il modello di business.

## Localizzazione

Coerente con la decisione già presa (`abiogenesis-cross-cutting.md`): lancio in **inglese soltanto**. La scelta di tenere i frammenti narrativi come dati strutturati invece che stringhe pre-concatenate (già presa per questo motivo) paga qui — se in futuro servirà localizzare, l'architettura non va riscritta da zero.

## Cosa serve per l'integrazione

- **Funzione di esportazione del riassunto di mondo**, prioritaria rispetto ad altre rifiniture di marketing — dipende solo da dati già calcolati (bilancio di fine mondo, Cronaca), non da nuovi sistemi.
- **Pagina itch.io** aperta presto, prima che il gioco sia completo, per cominciare a costruire pubblico durante lo sviluppo.
- **Roadmap pubblica per Steam Early Access**, derivata dalla classificazione a tre livelli già scritta in `abiogenesis-system-hierarchy.md` — non va scritta da zero, va tradotta in linguaggio per il pubblico.

## Fuori scope

- Data di lancio, durata prevista dell'Early Access.
- Dettaglio della funzione di esportazione (formato immagine vs testo, dove compare nell'interfaccia).
- Materiale di marketing concreto (trailer, pagina store, screenshot) — qui solo la strategia, non gli asset.
