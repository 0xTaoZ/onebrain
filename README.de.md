<p align="center">
  <img src="docs/assets/cover.webp" width="220" alt="OneBrain: Pixel-Agents rund um ein gemeinsames Gehirn">
</p>

<h1 align="center">OneBrain</h1>

<p align="center">
  <b>Ein Team aus Coding-Agents mit einem gemeinsamen Gehirn.</b><br>
  Es arbeitet nur der Agent, den du erwähnst, und er liest nur, was er noch nicht kennt.
</p>

<p align="center">
  <img alt="License Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-7c5cff?style=flat-square">
  <img alt="macOS" src="https://img.shields.io/badge/platform-macOS-1d1d1f?style=flat-square&logo=apple">
  <img alt="Rust and Tauri" src="https://img.shields.io/badge/built%20with-Rust%20%C2%B7%20Tauri-b7410e?style=flat-square&logo=rust">
  <img alt="Tokens saved" src="https://img.shields.io/badge/tokens%20saved-31%E2%80%9381%25-2b7d4a?style=flat-square">
</p>

<p align="center">
  <a href="README.md">English</a> · <a href="README.zh-CN.md">简体中文</a> · Deutsch
</p>

<p align="center">
  <img src="docs/assets/token-chart-de.svg" width="820" alt="Tokens pro Runde bleiben mit OneBrain konstant, beim erneuten Lesen des Verlaufs steigen sie; Ersparnis von 31 bis 81 Prozent">
</p>

## Was anders ist

|  | Übliches Multi-Agent-Setup | OneBrain |
| --- | --- | --- |
| Ein Agent holt auf | liest den Verlauf erneut | liest nur, was für ihn neu ist |
| Ein neuer Agent kommt dazu | bekommt das ganze Transkript | bekommt eine kurze Team-Übergabe |
| Wer arbeitet | alle, oder ein Orchestrator entscheidet | nur der Agent, den du erwähnst |
| Gedächtnis | steckt in jedem einzelnen Agent | ein gemeinsames Gehirn, das Menschen lesen können |
| Du änderst mitten im Lauf deine Meinung | die Kette läuft weiter | Übergaben warten auf dich |
| Kosten einer Runde | wachsen mit dem Gespräch | bleiben bei rund 1.000 Tokens |

## Ergebnisse

Drei Agents (Entwicklung, Review, Test) in einem Gespräch, pro Runde wird einer erwähnt. Beide Spalten geben den Agents dieselben Informationen.

| Gesprächslänge | Verlauf neu lesen | OneBrain | Ersparnis |
| ---: | ---: | ---: | ---: |
| 10 Runden | 18.752 | 12.859 | **31 %** |
| 30 Runden | 85.208 | 37.135 | **56 %** |
| 60 Runden | 258.464 | 72.928 | **72 %** |
| 100 Runden | 633.264 | 123.388 | **81 %** |

Ein Agent, der spät dazukommt, etwa ein zweiter Reviewer nach 100 Runden, startet mit 1.611 statt 11.364 Tokens (**86 %** weniger).

Alle Zahlen stammen aus einem einzigen Befehl, der ein festes Gespräch durch den echten Prompt-Builder schickt:

```sh
BENCH_TURNS=30 cargo test -p services --lib context_token_benchmark -- --ignored --nocapture
```

Methode und alle Werte: [docs/benchmarks/context-tokens.md](docs/benchmarks/context-tokens.md)

## Wie die Ersparnis entsteht

Agents verschiedener Anbieter können keinen gemeinsamen Provider-Cache nutzen. Deshalb reicht OneBrain keine Transkripte herum. Jeder Agent behält sein eigenes Gespräch, und zwischen den Agents wandern nur kleine, stabile Bausteine.

**Erwähnen, dann arbeiten.** Ein Agent läuft nur, wenn jemand ihn erwähnt. Wer wartet, kostet nichts.

**Ein Lesezeichen pro Agent.** Ein Agent, der sein Gespräch fortsetzt, bekommt nur die Nachrichten seit seinem letzten Lauf. Rolle und Teamregeln werden nicht erneut geschickt, solange sie gleich bleiben. Das Lesezeichen rückt erst nach einem erfolgreichen Lauf vor, ein fehlgeschlagener Lauf lässt also keine Nachrichten verschwinden.

**Team-Übergaben.** Ein Agent, der neu im Gespräch ist, bekommt deine letzten Anfragen, das letzte Ergebnis und die geänderten Dateien der anderen Agents sowie die bisher festgehaltenen Entscheidungen. Alles stammt aus Daten, die die Sitzung ohnehin speichert. Für eine Zusammenfassung wird kein zusätzlicher Modellaufruf verbraucht.

**Übergaben warten auf dich.** Schreibst du, während ein Agent arbeitet, werden seine Übergaben an andere Agents angezeigt, wecken sie aber nicht. Die Kette hält bei deiner neuen Nachricht an, statt mit alten Vorgaben weiterzulaufen.

**Cache-Trefferquote bei jeder Antwort.** Unter jeder Antwort stehen neue Eingabe, Cache-Treffer und Ausgabe, dazu der Anteil aus dem Provider-Cache. Anbieter zählen Cache-Tokens unterschiedlich; OneBrain vereinheitlicht die Werte, bevor es eine Quote zeigt.

## Das gemeinsame Gehirn

Die meisten Agent-Werkzeuge behalten ihr Wissen im einzelnen Agent: ein langer Chatverlauf oder Gedächtnisdateien, die nur dieser Agent liest. OneBrain führt ein Gedächtnis für das ganze Team, geschrieben für Menschen genauso wie für Agents.

Der Entwurf:

- Entscheidungen, Regeln, Erkenntnisse und Übergaben sind einfache Markdown-Dateien. Jede Agent-CLI kann sie öffnen, du kannst sie lesen, und git zeigt, wie sie sich verändert haben.
- Jeder Eintrag behält seine Quelle: die Nachricht, aus der er stammt, oder die Datei und Zeilen, die er beschreibt. Ändern sich diese Dateien, wird der Eintrag als veraltet markiert.
- Agents schlagen Einträge vor, du gibst sie frei. Nichts wird Teamwissen, ohne dass ein Mensch es gesehen hat.

Schon verfügbar: gemeinsame Einträge, Team-Übergaben und ein Lesezeichen pro Agent. In Arbeit: eine Seite in der App, um Einträge zu durchsuchen, freizugeben und zu bearbeiten.

Die Methode war zuerst da, die App kam danach. Mehrere Coding-Agents auf einem Rechner wurden schon vorher über gemeinsame Markdown-Dateien und schriftliche Regeln auf demselben Stand gehalten. OneBrain macht aus dieser Arbeitsweise ein Produkt.

## Außerdem in der App

- Gruppenchat in warmem Weiß, mit Pixel-Avataren für jede Agent-Laufzeit und einer Heatmap im GitHub-Stil für die täglichen Läufe jedes Agents.
- Das Backend beendet sich mit der App, auch nach einem Absturz oder erzwungenen Beenden, und stoppt die Agent-Prozesse, die es gestartet hat.
- Auf Wunsch ein eigener Git-Worktree pro Gespräch, damit Agents nie in deine Arbeitskopie schreiben.

Funktioniert mit Claude Code, Codex, Gemini CLI, Qwen Code, OpenCode, Amp und weiteren.

## Aus dem Quellcode bauen

Rust (Axum, SQLx, SQLite), React und TypeScript, Tauri.

```sh
pnpm install
pnpm run check
cargo test -p db -p services
```

## Danksagung

Apache License 2.0. Basiert auf [OpenTeams](https://github.com/openteams-lab/openteams), ebenfalls Apache-2.0; siehe [NOTICE](NOTICE).
