# Probleme während der Implementierung von Rusty Elevator

- Möglicherweise unzureichende Planung
    - Haben relativ früh mit dem konkreten Coding angefangen. 
    - Keine umfangreichen Diagramme.
- Komplexität der Anforderungen
    - Abdeckung aller Anforderungen führt ständig zu neuen Problemen.
    - Immer noch unzureichende Erfahrungswerte in Rust
    - Viele Konzepte in der kurzen Zeit noch abstrakt. 
- Modellierung der States
    - Wofür werden welche States benötigt?
    - Wie detailliert muss modelliert werden?
- Borrowing 
    - Verändern von Attributwerten
    - Rust erlaubt kein "double borrowing"
- Testing
    - Testfälle sind der anspruchsvoll durch komplexes Szenario.
    - Framework für das Testing wäre eigentlich angebracht.

## Entscheidungen im Design

- Wir haben uns gegen die Verwendung von Threads entschieden
    - Aufgrund zeitlicher Constraints
    - Sowie fehlender Erfahrung

- Stark hierarchische Struktur
    - Wir haben eine Art Kontrollsystem als übergeordnete Entität implementiert
    - Steuert die Logik, insbesondere von Interaktion zwischen Passagieren und Aufzügen
    - Beispielsweise das Erzeugen von Passagieren und zuweisen zu den Fahrstühlen

- Schrittweise Simulation durch "step" Methoden
    - Simulation wird so reproduzierbarer
    - Klare Schnittstellen für die Simulation
    - Klare Reihenfolge in der Simulation: Gamelogic -> Elevator
    - Sinnvolles Pattern für die "Game-Loop" der Visualisierung

- Optimierung der Steuerung indem Ziele beispielsweise in eigener Methode sortiert werden
    - Fährt alle Ziele in einer Richtung ab
    - Richtungswechsel werden möglichst vermieden
    - Sinnvoll hinsichtlich zeitlicher Optimierungen und Fairness