<p align="center">
  <img src="frontend/public/logos/onebrain-logo.png" width="88" alt="OneBrain-App-Symbol">
</p>

<h1 align="center">OneBrain Rooms</h1>

<p align="center">
  <b>Claude schreibt. Codex prüft. Ein Gruppenchat.</b><br>
  Open-Source-Gruppenchats für dich und deine Agents auf macOS. Tauscht euch aus, arbeitet gemeinsam an Projekten und haltet Entscheidungen in versioniertem Markdown fest.
</p>

<p align="center">
  <a href="https://github.com/0xTaoZ/onebrain/releases/latest"><b>Download für Apple Silicon</b></a>
  · <a href="#installation">Installation</a>
  · <a href="#arbeitsabläufe-im-vergleich">Vergleich</a>
  · <a href="#die-zahlen">Token-Benchmark</a>
</p>

<p align="center"><a href="README.md">English</a> · <a href="README.zh-CN.md">简体中文</a> · Deutsch</p>

<p align="center">
  <a href="docs/assets/demo/onebrain-demo.mp4">
    <img src="docs/assets/demo/onebrain-demo.gif" width="920" alt="OneBrain-Demo: Claude baut die automatische Speicherung, Codex entdeckt den Fehler und Gemini hält die Korrektur in Markdown fest">
  </a><br>
  <sub>Aufzeichnung der OneBrain-Oberfläche mit Beispieldaten.</sub>
</p>

<p align="center">
  <a href="docs/assets/demo/onebrain-demo.mp4">Demo als MP4 (Englisch)</a>
  · <a href="docs/assets/demo/group-chat.png">Gruppenchat als Bild</a>
  · <a href="docs/assets/demo/team-brain.png">Teamgedächtnis als Bild</a>
</p>

## Warum es OneBrain gibt

Ich wollte mehrere Agents zu einer Frage hinzuholen und am nächsten Tag dort weiterarbeiten, wo wir aufgehört hatten. Bei Code-Reviews kopierte ich ständig Anforderungen, Diffs und Antworten zwischen Terminals. Wenn der Review etwas fand, musste ich das Ergebnis wieder zurückkopieren.

In einem Chat kann ich mehrere Sichtweisen einholen. In einem Projektraum gebe ich den Agents einen Arbeitsordner, lasse eine Änderung durch ein anderes Modell prüfen und entscheide anhand des Diffs. Was wir dabei entscheiden, bleibt als Markdown für den nächsten Agent erhalten.

**Unser Ziel ist ein möglichst brauchbarer und sparsamer Arbeitsbereich für mehrere Agents.** Den Token-Verbrauch messen wir mit Gesprächen, die sich reproduzieren lassen.

## Was OneBrain Rooms auszeichnet

- **Code-Review über Anbietergrenzen hinweg.** Gib den nächsten Schritt per @ an Claude Code, Codex oder Gemini. In Projektgruppen siehst du Diffs und Freigaben neben dem Chat. Snapshots machen Dateiänderungen rückgängig und zeigen spätere Bearbeitungen vor dem Überschreiben an.
- **Kontext pro Agent.** Wer zurückkehrt, erhält neue Nachrichten. Neue Reviewer bekommen eine kurze Übergabe aus vorhandenen Daten. Im festen Replay mit 100 Runden wurden **80,9 % weniger Eingabe-Tokens** als beim erneuten Lesen des gesamten Verlaufs gezählt. [Benchmark wiederholen](docs/benchmarks/context-tokens.md).
- **Gemeinsames Markdown.** Die Coding-CLIs können die Dateien mit Teamentscheidungen direkt lesen. Git speichert Autor und Verlauf und ermöglicht das Wiederherstellen alter Versionen.

## Installation

Die App erscheint unter **OneBrain** im Ordner „Programme“.

| | Installation per Einzeiler (empfohlen) | `.dmg` herunterladen |
| --- | --- | --- |
| **So geht's** | Im Terminal einfügen:<br>`curl -fsSL https://raw.githubusercontent.com/0xTaoZ/onebrain/main/install.sh \| bash` | `OneBrain_x.y.z_aarch64.dmg` unter [Releases](https://github.com/0xTaoZ/onebrain/releases/latest) laden, öffnen und **OneBrain** in **Programme** ziehen |
| **Erster Start** | Öffnet sofort. Das Skript installiert die App und startet sie. | Falls macOS die heruntergeladene App blockiert, unter **Systemeinstellungen → Datenschutz & Sicherheit** auf **Trotzdem öffnen** klicken. Meldet macOS, die App sei beschädigt: `xattr -dr com.apple.quarantine /Applications/OneBrain.app` |
| **Voraussetzung** | Mac mit Apple Silicon (M1 oder neuer) | Ebenso |

Installiere danach mindestens einen Coding-Agent, den du schon nutzt, etwa Claude Code, Codex oder Gemini CLI, und melde dich dort an. Anmeldung und Abrechnung laufen über die konfigurierte CLI oder den Anbieter. Lege eine Gruppe an, füge deine Agents hinzu und erwähne einen mit `@`.

## Erster Chat

1. Wähle auf der Startseite einen erkannten Agent und klicke auf **Hinzufügen**.
2. Wähle eine Chatgruppe oder eine Projektgruppe. Eine Projektgruppe braucht auch einen Arbeitsordner.
3. Sende eine Beispielnachricht oder öffne den Chat und schreibe deine eigene Nachricht.

## Die Zahlen

<p align="center">
  <img src="docs/assets/token-chart-de.svg" width="840" alt="Kumulierte Eingabe-Tokens nach 10, 30, 60 und 100 Runden. Nach 100 Runden 649.322 mit vollständigem Verlauf und 123.829 mit OneBrain. 80,9 % weniger gegenüber dem Verlauf und 22,8 % weniger gegenüber nur Prompt.">
</p>

Festes Gespräch mit drei Agents und einer Erwähnung pro Runde. Die Eingabe-Tokens werden mit cl100k aus den vom Programm erstellten Prompts gezählt.

| Runden | Prompt + Verlauf erneut lesen | OneBrain | Ersparnis mit Verlauf | Ersparnis nur mit Prompt |
| ---: | ---: | ---: | ---: | ---: |
| 10 | 20.369 | 13.349 | **34,5 %** | 16,8 % |
| 30 | 90.014 | 37.594 | **58,2 %** | 21,8 % |
| 60 | 268.112 | 73.399 | **72,6 %** | 23,6 % |
| 100 | 649.322 | 123.829 | **80,9 %** | 22,8 % |

Ein Reviewer, der nach 100 Runden dazukommt, startet mit 1.775 Eingabe-Tokens. Beim erneuten Lesen des Verlaufs wären es 11.528. Die Zahlen stammen aus dem Benchmark; die Abrechnung hängt vom Anbieter ab.

[Methode](docs/benchmarks/context-tokens.md)

## Projektsteuerung

- **Arbeitsmodus.** Erstelle aus der Diskussion einen Plan, verteile Aufgaben und prüfe das Ergebnis.
- **Projekt-Worktrees.** Nutze für die Änderungen einer Gruppe einen separaten Git-Worktree und sieh dir den Diff im Chat an.
- **Zeitpläne und Aktivitäten.** Plane Gruppennachrichten und prüfe Aktivitätsprotokoll, Freigaben und Dateiänderungen.

## Arbeitsabläufe im Vergleich

| Projekt | Betrieb | Claude + Codex | Dateien zurücksetzen | Gemeinsames Markdown in Git | Leseposition pro Agent | Reproduzierbarer Token-Benchmark |
| --- | --- | :---: | :---: | :---: | :---: | :---: |
| **OneBrain Rooms** | Lokale App | ✅ | ✅ | ✅ | ✅ | ✅ |
| [Pachat](https://pachat.ai/use-cases/agent-teams) | Gehosteter Chat | ✅ | — | — | — | — |
| [Octo](https://www.mininglamp.com/octo/) | Selbst hostbar | [◐](https://github.com/Mininglamp-OSS/octo-runtime) | — | — | — | — |
| [OpenAgents](https://openagents.org/faq) | Selbst hostbar | ✅ | — | — | — | — |
| [AionUi](https://github.com/iOfficeAI/AionUi#team-mode--coordinated-multi-agent-collaboration) | Lokale App | ✅ | — | — | — | — |
| [PiBot](https://pibot.space/en/guides/bots-and-roles/#external-agents) | Lokale App | ✅ | [✅ macOS](https://github.com/w01fb0ss/PiBot#what-it-does) | — | — | — |
| [LobeHub](https://github.com/lobehub/lobehub) | Selbst hostbar | ✅ | — | — | — | — |
| [Claude Code Agent Teams](https://code.claude.com/docs/en/agent-teams) | Lokale CLI | ✗ | — | — | — | — |

**✅** dokumentiert · **◐** für die Laufzeit dokumentiert · **✗** außerhalb des dokumentierten Produktumfangs · **—** in den verlinkten Unterlagen kein entsprechendes Merkmal gefunden.

„Claude + Codex“ bedeutet, dass beide demselben Arbeitsbereich beitreten können. „Dateien zurücksetzen“ bezieht sich auf Änderungen durch einen Agent. Der [Token-Benchmark](docs/benchmarks/context-tokens.md) verwendet unser festes Gesprächs-Replay.

## Wie die Technik funktioniert

| Mechanismus | Umsetzung |
| --- | --- |
| [Inkrementelle Prompts](crates/services/src/services/chat_runner/prompting.rs) | Jeder Agent hat eine eigene Leseposition. Bei der Fortsetzung prüft der Runner den stabilen Präfix und ermittelt neue Nachrichten. Neue Mitglieder können mit einer Übergabe aus vorhandenen Daten beginnen. |
| [Teamgedächtnis](crates/services/src/services/brain.rs) | Entscheidungen liegen in Markdown. Git speichert Änderungen und Autoren. Ältere Stände lassen sich in der App öffnen. |
| [Git-Snapshots](crates/git/src/checkpoint.rs) | Die Rückgängig-Funktion beschränkt sich auf die Dateien eines Laufs. Spätere Änderungen an diesen Dateien werden vor dem Schreiben auf Konflikte geprüft. |
| [Zeitpläne](crates/services/src/services/scheduler/mod.rs) | Der Scheduler beansprucht fällige Aufgaben und sendet sie über den üblichen Chat-Pfad mit den normalen Freigaberegeln. |

## Wie wir KI einsetzen wollen

Ich schaue mir an, wie Leute, die KI-Werkzeuge bauen, sie selbst einsetzen. Eine Idee aus Lauren Tans [pstack-Anleitung](https://github.com/cursor/plugins/blob/main/pstack/docs/guide/README.md) gefällt mir besonders. Vor dem Start klären wir, woran wir erkennen, dass die Aufgabe fertig ist. Ein Agent reproduziert und behebt den Fehler. Ein zweiter prüft den Diff und führt den zuvor fehlschlagenden Test erneut aus. Die Entscheidung halten wir fest.

Im Gruppenchat möchte ich sehen, wer die Aufgabe übernommen hat, was sich geändert hat und warum der Review durchging. [brainmaxxing](https://github.com/poteto/brainmaxxing) zeigt, wie Wissen in bearbeitbaren Dateien erhalten bleibt. [Noodle](https://github.com/poteto/noodle) organisiert wiederkehrende Abläufe über Skills. Wenn ich nur eine zweite Meinung brauche, möchte ich einen Agent mit dem Diff und den relevanten Nachrichten beauftragen. Dann will ich sehen, welchen Test er ausgeführt hat und was dabei herauskam.

## Wie die Ersparnis entsteht

OneBrain merkt sich, was jeder Agent bereits gesehen hat. So bekommen Mitglieder neue Nachrichten und neu hinzukommende Agents eine kurze Übergabe.

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

## Dem Projekt folgen

Gib OneBrain Rooms einen Stern, wenn du die Entwicklung verfolgen möchtest. Berichte gern in den [GitHub Issues](https://github.com/0xTaoZ/onebrain/issues) von deinem ersten Einsatz.

---

<sub>Apache License 2.0. Basiert auf OpenTeams (Apache-2.0); siehe NOTICE.</sub>
