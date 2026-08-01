# Task [ID] — [Titolo del Task]

> **ID**: `[NNN]`
> **Categoria**: [Architettura / Feature / Bugfix / Refactor / etc.]
> **Priorità**: [🔴 P1 / 🟡 P2 / 🟢 P3]
> **Stima**: [~1h / ~2h / etc.]
> **Assegnato a**: [Claude CLI / non assegnato]
> **Sessione**: [ID conversazione o riferimento temporale]

---

## 🎯 Obiettivo

[Cosa deve essere fatto?]
[Perché è necessario?]

---

## 📋 Acceptance Criteria

[Un task è considerato completato quando:]
- [ ] Il codice compila senza errori.
- [ ] La feature X funziona come descritto.
- [ ] [Aggiungere criteri specifici...]

---

## 📁 File Rilevanti

| File | Ruolo |
|------|-------|
| `src/modulo/file.ts` | Descrizione del ruolo. |

---

## 🧩 Contesto Tecnico

[Incolla qui le definizioni di tipi, interfacce, o descrive lo stato attuale del codice.]

- **Comportamento attuale**: [Cosa succede ora?]
- **Comportamento desiderato**: [Cosa deve succedere dopo?]

---

## 🔨 Implementazione Suggerita

[Passaggi consigliati per l'agente IA]

1. [Passo 1]
2. [Passo 2]

```
// Eventuale snippet di esempio
```

---

## ⚠️ Vincoli e Attenzioni

- **Stile**: Seguire le convenzioni definite nel `TECH_DESIGN.md`.
- **Performance**: [Eventuali vincoli specifici]

---

## 🔗 Dipendenze

- **Dipende da**: [ID task precedente o nessuno]
- **Blocca**: [ID task successivo o nessuno]

---

## 🤖 Come delegare questo task a Claude CLI

```bash
claude "$(cat tasks/[NNN]-nome.md)"$'\n\nEsegui questo task nel progetto corrente.'
```
