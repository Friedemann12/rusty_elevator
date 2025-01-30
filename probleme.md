# Probleme während der Implementierung von Rusty Elevator


- Borrowing 
    - Verändern von Attributwerten
    - Rust erlaubt kein "double borrowing"
- Komplexität der Anforderungen
    - Abdeckung aller Anforderungen führt ständig zu neuen Problemen.
- Testing
    - Testfälle sind der anspruchsvoll durch komplexes Szenario.
    - Framework für das Testing wäre eigentlich angebracht.
- Modellierung der States
    - Wofür werden welche States benötigt?
    - Wie detailiert muss modelliert werden?

## Entscheidungen im Design

- Wir haben uns gegen die Verwendung von Threads entschieden
    - Aufgrund zeitlicher Constraints
    - Sowie fehlender Erfahrung


- Stark hierarchische Struktur
    - Wir haben eine Art ControlSystem als übergeordnete Entität implementiert
    - Steuert die Logik, insbesondere von Interaktion zwischen Passagieren und Aufzügen
    - Beispielsweise das Erzeugen von Passagieren und zuweisen zu den Fahrstühlen

- Schrittweise Simulation durch "step" Methoden
    - Simulation wird so reproduzierbarer
    - Klare Schnittstellen für die Simulation
    - Klare Reihenfolge in der Simulation: Gamelogic -> Elevator
    - Sinnvolles Pattern für die "Game-Loop" der Visualisierung

- Optimierung der Steuerung indem Ziele beispielsweise sortiert werden
    - Fährt alle Ziele in einer Richtung ab
    - Richtungswechsel werden möglichst vermieden
    - Sinnvoll hinsichtlich zeitlicher Optimierungen und Fairness