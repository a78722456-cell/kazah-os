# ADR-0002: KazIPC Design

## Status
Accepted

## Context
Inter-Process Communication (IPC) is the lifeblood of a micro-hybrid operating system. Without an efficient, robust messaging protocol, communication between system services (ServiceManager, ProcessManager, DeviceManager, DriverManager) becomes a bottleneck.

## Decision
KAZAH OS implements **KazIPC**, an asynchronous, object-based communication mechanism integrated directly into the Kernel Object Manager (KOM):
1. **Channel Objects**: Communication occurs over `ChannelObject` endpoints registered in process handle tables.
2. **Syscall Routing**: Application-level device APIs (`OpenDevice`, `ReadDevice`, `WriteDevice`, `Ioctl`) are exposed as system calls (`Syscall 13..16`) that internally route messages over IPC channels to the respective driver services.
3. **Message Framing**: Standardized message payloads (`StorageRequest`, `BindDeviceMsg`) ensure strict type safety across process boundaries.

## Consequences
- **Positive**: Unified API surface for applications—reading from a storage device looks identical to reading from a network socket or file.
- **Positive**: Transparent message interception enables universal audit logging and time-travel debugging.
