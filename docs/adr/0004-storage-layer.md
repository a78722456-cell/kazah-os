# ADR-0004: Storage Layer & Block Device Abstraction

## Status
Accepted

## Context
To support diverse physical and synthetic storage technologies (SATA AHCI, PCIe NVMe, RAM Disks, VirtIO), the OS cannot couple higher-level filesystems directly to specific driver implementations.

## Decision
KAZAH OS introduces the **Storage Driver Framework**:
1. **`BlockDevice` Trait**: All block drivers implement a universal trait defining `read_blocks`, `write_blocks`, `flush`, `block_size`, and `block_count`.
2. **`StorageObject` Representation**: Block storage instances are represented in the Kernel Object Manager as `ObjectType::Storage`, tracked inside `STORAGE_REGISTRY`.
3. **Asynchronous IO Queues**: Drivers process IO operations asynchronously via `StorageRequest` IPC messages (`Read`, `Write`, `Flush`, `Identify`), laying the groundwork for multi-queue NVMe architectures.
4. **RAM Disk Prototyping**: A native `ramdisk` userspace driver validates storage protocols and filesystems independent of hardware controller idiosyncrasies.

## Consequences
- **Positive**: Filesystems (VFS/KFS) interact exclusively with abstract block devices, decoupling logic from hardware.
- **Positive**: Seamless integration into the Universal State Engine via `StorageSnapshot`.
