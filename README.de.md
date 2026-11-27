<p align="center">
  <img src="docs/assets/cover.webp" width="200" alt="OneBrain: Pixel-Agents rund um ein gemeinsames Gehirn">
</p>

<h1 align="center">OneBrain</h1>

<p align="center">
  <b>Deine Coding-Agents in einem Gruppenchat, gebaut für Entwickler.</b><br>
  Claude Code, Codex und Gemini reden auf deinem Mac miteinander, planen gemeinsam und prüfen gegenseitig ihren Code.<br>
  Im Benchmark mit 100 Runden sinken die Eingabe-Tokens gegenüber erneutem Lesen des Verlaufs um 80,9 %.
</p>

<p align="center">
  <b>80,9 % weniger Eingabe-Tokens</b> gegenüber erneutem Lesen des Verlaufs im Benchmark mit 100 Runden
</p>

<p align="center">
  <a href="#installation"><b>⬇ Für macOS installieren</b></a>
  &nbsp;·&nbsp; <a href="README.md">English</a> · <a href="README.zh-CN.md">简体中文</a> · Deutsch
</p>

<p align="center">
  <img alt="License Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-7c5cff?style=flat-square">
  <img alt="macOS Apple Silicon" src="https://img.shields.io/badge/macOS-Apple%20Silicon-1d1d1f?style=flat-square&logo=apple">
  <img alt="Lokal, ohne Konto" src="https://img.shields.io/badge/l%C3%A4uft-lokal%2C%20ohne%20Konto-2b7d4a?style=flat-square">
  <img alt="Release" src="https://img.shields.io/github/v/release/0xTaoZ/onebrain?style=flat-square&color=7c5cff">
</p>

## Drei Arten, es zu nutzen

**Chat-Modus.** Ein Gruppenchat wie jeder andere. Erwähnst du einen Agent mit `@`, antwortet nur dieser. Claude Code baut, Codex prüft, Gemini schreibt die Doku, alles im selben Verlauf.

**Arbeitsmodus.** Aus einer Diskussion wird ein Plan. Der leitende Agent schreibt die Schritte, jeder Schritt läuft mit dem Agent, den du zugeteilt hast, und erst wenn du das Ergebnis abnimmst, ist er erledigt.

**Für Entwickler gebaut.** Jedes Gespräch kann einen eigenen Git-Worktree bekommen, damit Agents nie in deine Arbeitskopie schreiben. Diffs, geänderte Dateien, Freigaben und Versionskontrolle liegen direkt neben dem Chat, und jede Antwort zeigt ihren Token-Verbrauch und die Cache-Trefferquote.

## Installation

| | Installation per Einzeiler (empfohlen) | `.dmg` herunterladen |
| --- | --- | --- |
| **So geht's** | Im Terminal einfügen:<br>`curl -fsSL https://raw.githubusercontent.com/0xTaoZ/onebrain/main/install.sh \| bash` | `OneBrain_x.y.z_aarch64.dmg` unter [Releases](https://github.com/0xTaoZ/onebrain/releases/latest) laden, öffnen und **OneBrain** in **Programme** ziehen |
| **Erster Start** | Öffnet sofort. Das Skript installiert die App und startet sie. | macOS blockiert Apps ohne bezahltes Apple-Entwicklerkonto. Unter **Systemeinstellungen → Datenschutz & Sicherheit** auf **Trotzdem öffnen** klicken. Meldet macOS, die App sei beschädigt: `xattr -dr com.apple.quarantine /Applications/OneBrain.app` |
| **Voraussetzung** | Mac mit Apple Silicon (M1 oder neuer) | Ebenso |

Installiere danach mindestens einen Coding-Agent, den du schon nutzt, etwa Claude Code, Codex oder Gemini CLI, und melde dich dort an. OneBrain nutzt deine eigenen Abos und sieht deine Schlüssel nie. Lege eine Gruppe an, füge deine Agents hinzu und erwähne einen mit `@`.

## Erster Chat

1. Wähle auf der Startseite einen erkannten Agent und klicke auf **Hinzufügen**.
2. Wähle eine Chatgruppe oder eine Projektgruppe. Eine Projektgruppe braucht auch einen Arbeitsordner.
3. Sende eine Beispielnachricht oder öffne den Chat und schreibe deine eigene Nachricht.

## Im Vergleich

|  | OneBrain | Session-Manager<br><sub>Claude Squad, Conductor</sub> | Gruppenchats in der Cloud<br><sub>xMatrix, Pachat</sub> | Teams eines Anbieters<br><sub>Claude Code Agent Teams, Antigravity /swarm</sub> |
| --- | :---: | :---: | :---: | :---: |
| Agents verschiedener Anbieter reden in einem Chat | ✅ | – | ✅ | – |
| Läuft auf deinem Mac, ohne zusätzliches Konto | ✅ | ✅ | – | ✅ |
| Wer später dazukommt, bekommt eine kurze Übergabe statt des Verlaufs | ✅ | – | – | teilweise |
| Gemeinsames Teamgedächtnis, das Menschen lesen können | ✅ | – | ✅ | – |
| Veröffentlichte, nachprüfbare Token-Messungen | ✅ | – | – | – |

## Warum es OneBrain gibt

Ich arbeite mit mehreren Coding-Agents gleichzeitig. Der größte Teil der Kosten ging dafür drauf, dass sie denselben Chatverlauf immer wieder lasen, und damit sie sich Arbeit übergeben konnten, musste ich Text zwischen Terminals hin und her kopieren. OneBrain bringt sie in einen Chat und gibt jedem nur das, was er braucht.

## Die Zahlen

Festes Gespräch mit drei Agents und einer Erwähnung pro Runde. Die Eingabe-Tokens werden mit cl100k aus den vom Programm erstellten Prompts gezählt.

| Runden | Prompt + Verlauf erneut lesen | OneBrain | Ersparnis mit Verlauf | Ersparnis nur mit Prompt |
| ---: | ---: | ---: | ---: | ---: |
| 10 | 20.369 | 13.349 | **34,5 %** | 16,8 % |
| 30 | 90.014 | 37.594 | **58,2 %** | 21,8 % |
| 60 | 268.112 | 73.399 | **72,6 %** | 23,6 % |
| 100 | 649.322 | 123.829 | **80,9 %** | 22,8 % |

Ein Reviewer, der nach 100 Runden dazukommt, startet mit 1.775 Eingabe-Tokens. Beim erneuten Lesen des Verlaufs wären es 11.528. Die Zahlen stammen aus dem Benchmark; die Abrechnung hängt vom Anbieter ab.

[Methode](docs/benchmarks/context-tokens.md)

## Wie die Ersparnis entsteht

Agents verschiedener Anbieter können keinen gemeinsamen Provider-Cache nutzen. Deshalb reicht OneBrain keine Transkripte herum. Jeder Agent behält sein eigenes Gespräch, und zwischen den Agents wandern nur kleine, stabile Bausteine.

- **Antwortende wählen.** Nenne einen Agent mit @ oder frage die Gruppe mit @all. Ohne Erwähnung antwortet ein ausgewählter Agent.
- **Ein Lesezeichen pro Agent.** Wer sein Gespräch fortsetzt, bekommt nur die Nachrichten seit seinem letzten Lauf. Rolle und Teamregeln werden nicht erneut geschickt, solange sie gleich bleiben.
- **Team-Übergaben.** Neue Agents bekommen deine letzten Anfragen, das letzte Ergebnis und die geänderten Dateien der anderen und die festgehaltenen Entscheidungen, aus gespeicherten Daten und ohne zusätzlichen Modellaufruf.
- **Übergaben warten auf dich.** Schreibst du, während ein Agent arbeitet, weckt seine Übergabe den nächsten Agent nicht. So arbeitet niemand mit Vorgaben weiter, die du gerade geändert hast.

## Das gemeinsame Gehirn

Die meisten Agent-Werkzeuge behalten ihr Wissen im einzelnen Agent. OneBrain führt ein Gedächtnis für das ganze Team: Entscheidungen, Regeln, Erkenntnisse und Übergaben als einfaches Markdown, das jeder Agent öffnen, du lesen und git verfolgen kann. Jeder Eintrag behält seine Quelle, Agents schlagen vor, du gibst frei.

Die Brain-Seite zeigt gemeinsame Einträge und Übergaben.

## Aus dem Quellcode bauen

Rust (Axum, SQLx, SQLite), React und TypeScript, Tauri.

```sh
pnpm install
pnpm run check
cargo test -p db -p services
```

---

<sub>Apache License 2.0. Basiert auf OpenTeams (Apache-2.0); siehe NOTICE.</sub>
