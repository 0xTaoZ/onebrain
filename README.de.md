<p align="center">
  <img src="frontend/public/logos/onebrain-logo.png" width="88" alt="OneBrain-App-Symbol">
</p>

<h1 align="center">OneBrain Rooms</h1>

<p align="center">
  <b>Claude schreibt. Codex prüft. Ein Gruppenchat.</b><br>
  Ein Gruppenchat für den Mac, für alle, die Claude Code, Codex und Gemini an derselben Arbeit einsetzen.<br>
  Sie übergeben sich Aufgaben und prüfen gegenseitig ihren Code, ohne jedes Mal den ganzen Verlauf zu lesen, und teilen ein Gedächtnis, das jeder Anbieter lesen kann.
</p>

<p align="center">
  <a href="https://github.com/0xTaoZ/onebrain/releases/latest"><b>Download für Apple Silicon</b></a>
  · <a href="#installation">Installation</a>
  · <a href="#arbeitsabläufe-im-vergleich">Vergleich</a>
  · <a href="#die-zahlen">Token-Benchmark</a>
</p>

<p align="center"><a href="README.md">English</a> · <a href="README.zh-CN.md">简体中文</a> · Deutsch</p>

<p align="center">
  <a href="https://github.com/0xTaoZ/onebrain/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/0xTaoZ/onebrain?style=flat-square&color=1d1d1f"></a>
  <img alt="macOS Apple Silicon" src="https://img.shields.io/badge/macOS-Apple%20Silicon-1d1d1f?style=flat-square&logo=apple">
  <a href="LICENSE"><img alt="License Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-1d1d1f?style=flat-square"></a>
</p>

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

## Was du bekommst

- **Claude, Codex und Gemini in einem Chat.** `@codex` fragt einen Agent, `@all` fragt alle, und eine Nachricht ohne Erwähnung geht an das passendste Mitglied. Übergaben passieren im selben Verlauf.
- **80,9 % weniger Input-Tokens in langen Gesprächen.** Jeder Agent hat ein Lesezeichen und bekommt nur neue Nachrichten; wer später dazukommt, erhält eine kurze Übergabe statt des Verlaufs. Gemessen an einem festen Gespräch mit 100 Runden; [selbst nachprüfen](docs/benchmarks/context-tokens.md).
- **Ein Gehirn, das jeder Anbieter lesen kann.** Entscheidungen und Regeln liegen in einem normalen Markdown-Ordner, den Claude Code, Codex und Gemini direkt öffnen. Git hält fest, welcher Agent was geschrieben hat, und jede Datei lässt sich auf einen älteren Stand zurücksetzen.
- **Sicher im Hintergrund.** Jeder Lauf lässt sich rückgängig machen, und vorher wirst du gewarnt, wenn du die Dateien inzwischen geändert hast. Befehle, die außerhalb des Projekts schreiben oder Historie umschreiben, werden in der Freigabe markiert und lassen sich nie dauerhaft erlauben. Geplante Läufe brauchen dieselben Freigaben.

## Installation

| | Installation per Einzeiler (empfohlen) | `.dmg` herunterladen |
| --- | --- | --- |
| **So geht's** | Im Terminal einfügen:<br>`curl -fsSL https://raw.githubusercontent.com/0xTaoZ/onebrain/main/install.sh \| bash` | `OneBrain_x.y.z_aarch64.dmg` unter [Releases](https://github.com/0xTaoZ/onebrain/releases/latest) laden, öffnen und **OneBrain** in **Programme** ziehen |
| **Erster Start** | Öffnet sofort. | Blockiert macOS die App: **Systemeinstellungen → Datenschutz & Sicherheit → Trotzdem öffnen**. Meldet macOS, die App sei beschädigt: `xattr -dr com.apple.quarantine /Applications/OneBrain.app` |
| **Voraussetzung** | Mac mit Apple Silicon (M1 oder neuer) und mindestens ein angemeldetes Coding-CLI: Claude Code, Codex oder Gemini CLI | Ebenso |

OneBrain nutzt die CLIs, die du schon hast, mit deinen eigenen Abos, und sieht deine Schlüssel nie. In **Programme** heißt die App **OneBrain**.

**Erster Chat:** Der Startbildschirm zeigt die Agents, die auf deinem Mac gefunden wurden. Lege eine Gruppe an (Chat-Gruppe oder Projektgruppe mit Code-Ordner) und klicke eine Beispielnachricht an oder schreib selbst.

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

## Das Team-Gehirn

Was ein Anbieter sich merkt, bleibt bei diesem Anbieter: Was Claude lernt, sieht Codex nie. OneBrain führt ein Gedächtnis für das ganze Team in `~/OneBrain/brain`, einem Ordner nach der `AGENTS.md`-Konvention. Es wird mit jeder Nutzung besser: Was ein Agent festhält, steht dem nächsten zur Verfügung, egal von welchem Anbieter er kommt.

- **Von den Agents gepflegt.** Sie lesen den Index, öffnen nur, was die Aufgabe braucht, und halten Entscheidungen, Regeln und Übergaben fest. Dafür ist kein zusätzlicher Modellaufruf nötig.
- **Jede Änderung ist signiert.** Der Ordner ist ein Git-Repository. Nach jedem Lauf werden die Änderungen mit dem Agent als Autor committet, jede Zeile lässt sich einem Agent und einem Lauf zuordnen.
- **Du siehst, was es kostet.** Die Gehirn-Seite zeigt, wie viele Tokens jeder Agent zum Lesen braucht, wer was geändert hat, und ältere Stände lassen sich mit einem Klick zurückholen.

## Wie die Technik funktioniert

```text
  Claude Code     Codex      Gemini CLI        deine eigenen CLIs und Abos
       │            │            │
       └────────────┼────────────┘
                    ▼
     ┌─────────────────────────────┐
     │       OneBrain Rooms        │   ein Gruppenchat auf deinem Mac
     │  read cursor per agent      │   jeder Agent bekommt nur Neues
     │  git snapshot per run       │   jeder Lauf ist umkehrbar
     └──────────────┬──────────────┘
                    ▼
     ~/OneBrain/brain  (Markdown + git)   ein Gedächtnis für alle Anbieter,
                                          jede Änderung vom Agent signiert
```

| Mechanismus | Umsetzung |
| --- | --- |
| [Inkrementelle Prompts](crates/services/src/services/chat_runner/prompting.rs) | Jeder Agent hat eine eigene Leseposition. Bei der Fortsetzung prüft der Runner den stabilen Präfix und ermittelt neue Nachrichten. Neue Mitglieder können mit einer Übergabe aus vorhandenen Daten beginnen. |
| [Team-Gehirn](crates/services/src/services/brain.rs) | Ein Markdown-Ordner unter Git. Nach jedem Lauf werden die Änderungen mit dem Agent als Autor committet; den Ort erfährt jeder Agent einmal pro Gespräch im gecachten Teil des Prompts. |
| [Git-Snapshots](crates/git/src/checkpoint.rs) | Die Rückgängig-Funktion beschränkt sich auf die Dateien eines Laufs. Spätere Änderungen an diesen Dateien werden vor dem Schreiben auf Konflikte geprüft. |
| [Zeitpläne](crates/services/src/services/scheduler/mod.rs) | Der Scheduler beansprucht fällige Aufgaben und sendet sie über den üblichen Chat-Pfad mit den normalen Freigaberegeln. |

## Warum es OneBrain gibt

Ich lasse mehrere Coding-Agents an derselben Aufgabe arbeiten. Der größte Teil der Kosten ging dafür drauf, denselben Verlauf immer wieder zu lesen, und für eine zweite Meinung musste ich Anfrage, Diff und letzte Antwort zwischen Terminals kopieren. OneBrain holt sie in einen Chat, gibt jedem nur, was er noch nicht gesehen hat, und hält fest, was das Team entscheidet, in Dateien, die der nächste Agent lesen kann.

Vorher habe ich mir angesehen, wie LobeHub, AionUi und PiBot Gedächtnis, Rückgängig-Machen und Zeitpläne lösen, und die Ideen, die mir gefielen, in Rust neu gebaut, mit möglichst wenigen Tokens. Die Idee, das Gedächtnis in normalen Dateien zu halten, die Agents selbst bearbeiten, stammt aus potetos [brainmaxxing](https://github.com/poteto/brainmaxxing).

## Aus dem Quellcode bauen

Rust (Axum, SQLx, SQLite), React und TypeScript, Tauri.

```sh
pnpm install
pnpm run check
cargo test --workspace
```

## Dem Projekt folgen

Gib OneBrain Rooms einen Stern, um die Entwicklung zu verfolgen, oder erzähl auf [GitHub Issues](https://github.com/0xTaoZ/onebrain/issues), was in deiner ersten Gruppe passiert ist.

---

<sub>Apache License 2.0. Basiert auf OpenTeams (Apache-2.0); siehe NOTICE.</sub>
