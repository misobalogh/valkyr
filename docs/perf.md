# Performance budgets

Initial targets. They are revised once the first benchmarks exist; a change to a
number needs a note explaining why.

| Area                      | Budget                                   |
|---------------------------|------------------------------------------|
| Sim step (server)         | under 4 ms per tick at 60 Hz, 8 players  |
| Sim step (client predict) | under 0.5 ms per local player step       |
| Entity count              | 5,000 simulated entities per server      |
| Bandwidth per player      | under 64 KiB/s down, under 8 KiB/s up    |
| Snapshot size             | under 1,200 bytes per packet after delta |

## Benchmarks

Sim hot paths get a criterion benchmark (`cargo bench -p sim`). The CI
regression gate is added together with the benchmark dependency.
