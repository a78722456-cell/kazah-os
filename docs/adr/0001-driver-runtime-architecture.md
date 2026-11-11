# ADR-0001: Driver Runtime Architecture

## Status
Accepted

## Context
Traditional monolithic kernels execute hardware drivers in Ring 0. A single bug or unhandled hardware exception in a storage or network driver can crash the entire operating system. Conversely, pure microkernels often suffer from extreme latency overhead due to excessive context switching when transferring data back and forth between userspace drivers and kernel space.

## Decision
KAZAH OS adopts a **Micro-Hybrid Driver Runtime**:
1. **Userspace Isolation**: All device drivers execute as isolated Ring 3 services managed by `driver_manager` (PID 3).
2. **Explicit Binding**: Drivers do not receive configuration via command-line spawn arguments. Instead, they register their IPC channels and receive explicit `BindDevice` messages containing target device IDs.
3. **Supervisor Recovery**: If a driver faults or crashes, the kernel updates its `DriverObject` state to `Faulted` and emits telemetry. The `driver_manager` detects process death, automatically re-spawns the driver, and re-sends the `BindDevice` handshake.
4. **Kernel Registry**: The kernel maintains a `DRIVER_REGISTRY` mapping hardware device IDs to driver process IDs, enabling fast lookups without querying userspace services on every operation.

## Consequences
- **Positive**: Complete fault isolation. A crashed driver does not bring down Ring 0.
- **Positive**: Hot-plugging and driver upgrades can happen seamlessly without rebooting the OS.
- **Negative**: Requires careful DMA memory mapping and IPC zero-copy optimization to match monolithic throughput.
