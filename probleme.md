# Probleme während der Implementierung von Rusty Elevator


- Borrowing 
    - Verändern von Attributwerten
    - Rust erlaubt kein "double borrowing"
- Komplexität der Anforderungen
    - Abdeckung aller Anforderungen führt ständig zu neuen Problemen
- Testing
    - Testfälle sind der anspruchsvoll durch komplexes Szenario
    - Framework für das Testing wäre eigentlich angebracht
- Modellierung der States
    - Für was werden welche States benötigt
    - Wie detailiert muss modelliert werden

- Wir haben uns gegen die Verwendung von Threads entschieden
    - Aufgrund zeitlicher Constraints
    - Und fehlender Erfahrung