# ADR-0003: Universal State Engine & Live Rollback

## Status
Accepted

## Context
Operating systems typically treat state as ephemeral memory constructs scattered across disparate kernel data structures. When an error occurs or system debugging is required, developers must rely on static core dumps or intrusive debuggers. Furthermore, recovering from service or driver failures usually necessitates system restarts.

## Decision
KAZAH OS establishes the **Universal State Engine** as its primary architectural differentiator:
1. **Everything has History**: Every kernel object (`Process`, `Thread`, `Device`, `Driver`, `Storage`, `Window`, `Socket`, `Volume`, `Mount`) records snapshots into a unified ring buffer (`HISTORIAN`).
2. **Standardized Snapshot Records**: The `SnapshotRecord` enum encapsulates snapshots from all subsystems, tagged with monotonic timestamps and object IDs.
3. **Live Rollback**: Because historical state transitions are immutably preserved, system objects (such as windows, drivers, or services) can be rolled back to prior functional states without triggering system reboots.

## Consequences
- **Positive**: Unprecedented observability and time-travel debugging capabilities.
- **Positive**: High system resiliency through instant rollback of corrupted or failed states.
- **Negative**: Requires bounded ring buffer allocations to prevent memory exhaustion during high-frequency telemetry emission.
