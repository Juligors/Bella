- Zobaczyć jak się dodaje rzeczy żeby były widoczne w tym bevy inspector, żeby nie debugować tego jakoś ręczie: IMPL
    - plus jak nazywać te entity na głównym poziomie jakoś lepiej niż te defaultowe, bo nie wiadomo co jest co: IMPL

- Dorobienie funkcji dla zwierząt: TODO
    - spadek głodu
    - dmg pod wpływem głodowania
    - zjedzenie roślinki lub zwierzęcia (dorobić componenty dla roślinorzercy/mięsorzercy) i wzrost nasycenia w związku z tym
    - może sen jeśli jest najedzone, wtedy wolnejszy spadek głodu
- Lepsze kolorki dla roślin i zwierząt, nie ciągłe, tylko raczej enumy: TODO

- bevy_pancam? BACKLOG
- Ustabilizować temperaturę i ogólnie umieranie organizmów: BACKLOG
- Roślinki niech rosną na ziemi i się rozmnarzają: BACKLOG
- Spawnowanie się roślin i zwierząt losowo wewnątrz mapy: BACKLOG

- zmienić podział plików, żeby było sensownie: BACKLOG
- Zobaczyć jak optymalizować, jakieś flamegraphs itp. BACKLOG
- zastanowić się co powinno być zrobione przy pomocy `system_set` a co z `state`: BACKLOG
- zmienić temperaturę z i32 na Temperature, gdzie new() zapewnia odpowiednie wartości: BACKLOG


## Continuous

Stosuj eventy i states

## Collisions
- ‌'separate axis theorem' for collisions?

## Terrain generation
- ‌wave function collapse for procedural generation (terrain? animals?)

## Systems
- movement consumes energy more than laying or sleeping
- ‌most "harbivores" are really "opportunistic omnivores", they eat meat when it's possible
- każdy organizm powinien mieć swoją krzywą Gaussa temperatur w których jest mu dobrze
‌- rośliny rozmnażają się przez nasiona?
‌- sieć neutonowa jako mózg, bardziej własna implementacja
- ‌ciepło i energia, im większy organizm tym więcej ciepła wydziela. wielkość zwierzęcia i stosunek masy do powierzchni - wydajność cieplna?
- ‌fenotyp i genotyp, przekazywanie genów: te takie silne i słabe, co oboje rodziców musi go mieć, geny uśpione
- ‌rozbudować roślinność o np. pożar, jakiś wpływ i zależność od wilgotności powietrza
- ‌stany: stan poruszania się: bezruch, chodzenie, bieg, pływanie w miejscu, pływanie w jakąś stronę

## Long term ideas/visions
- najpierw prosta symulacja fizyki typu temperatura, ukształtowanie terenu, wilgotność i opady deszczu, woda (to byłoby cool, tworzenie się nowych jezior itp.), a dopiero potem rośliny i na końcu zwierzęta, tak jak to było w rzeczywistości.

- ‌opcja na interakcje w trakcie symulacji, np. wrzucamy jedzenie między osobniki żeby zobaczyć jak zaregagują,albo np. naciskamy na ekran żeby wydać dźwięk, żeby je zwabić
- ‌nie zapisy całe, ale zapisywać jakieś najważniejsze info do analizy potem
- ‌kilka oddzielnych ekosystemów, które potem można połączyć (wyspy?)
- ‌phydoplancton? base of foodchain
- ‌convergent evolution

- ‌miejsca, gdzie ofiary mogą się schować, żeby nie wyginęły (to niekoniecznie, tylko jeśli będzie potrzeba), może w formie np. gęstego lasu z krzakami, gdzie króliki mogą się schować
- ‌duże kontrasty np. zimny ocean, niedaleko gorący wulkan, zacieniony las i gorąca pustynia
- ‌pogoda, zmiany temperatury w dłużym okresie np. epoka lodowcowa
- ‌oznaczenie osobnika, żeby można było śledzić jego i jego następców (możliwość oznaczenia wielu osobników wieloma "kolorami" jednocześnie)
- ‌(?) dwutlenek węgla wpływa na atmosferę, sprawiając, że jest cieplej. Czy to powinno być included?

## UI/Visuals
- F1 F2 etc. zmienia widok jak w Oxygen not included: F1 to widok terenu, F2 to widok temperatury itp. Przy pomocy stanu gry? albo jakiegoś Res? ALTERNATYWNIE ‌wyświetlanie info o Tile w formie interfejsu, najlepiej zwracającego Iterator z sekcjami, żeby różne rzeczy mogły mieć różną liczbę sekcji
- ‌zoomownie powinno zoomować do kursora, nie na środek ekranu
- ‌fullscreen i wielkość UI zależna od wielkości ekranu (da się w trakcie rozgrywki zmieniać wielkość np. przy pomocy myszki? czy bardziej ukrywanie/zmiana okien?)
- przycisk pauzy

## Others
- ‌wypisać sobie konwencje nazewnictwa i się ich trzymać
- ‌dokładniejszy research i decyzja co chcę zrobić i osiągnąć (yt, artykuły)
- ‌https://youtu.be/JQVmkDUkZT4?si=dyMFlXsIRNpszPXK
- ‌pomysł na ew. dodanie AI do magisterki: sieć neuronowa kontrolująca warunki, żeby np. jakiś gatunek (lub jak najwięcej gatunków) rozwinął się jak najbardziej, lub sieć neuronowa dla każdego gatunku
