# Getting Started with KAZAH OS

This guide walks you through setting up the development environment, compiling the operating system, and running it in an emulator (QEMU).

---

## 🛠️ Prerequisites

To build and run KAZAH OS, you need the following tools installed on your host system:

### 1. Rust Nightly Toolchain
KAZAH OS is written in modern Rust (2024 Edition) and relies on nightly features for bare-metal compilation (`build-std`, `json-target-spec`, `naked_functions`, etc.).

```bash
rustup default nightly
rustup component add rust-src llvm-tools-preview
```

### 2. Build Tools
- **PowerShell** (5.1+ or PowerShell Core 7+)
- **tar** (Available out-of-the-box on modern Windows 10/11 and Linux)
- **curl** (For fetching bootloader assets)

### 3. Emulation (QEMU)
To test and execute the OS:
- **QEMU for x86_64** (`qemu-system-x86_64`)
- **OVMF / UEFI Firmware** (Usually shipped with QEMU as `OVMF.fd` or `OVMF_CODE.fd`)

---

## 🚀 Building KAZAH OS

The repository provides an automated build script `build.ps1` that orchestrates:
1. Compiling all Userspace services (`service_manager`, `device_manager`, `driver_manager`, drivers, and test binaries).
2. Packing userspace binaries into a ramdisk archive (`initrd.tar`).
3. Compiling the Micro-Hybrid Kernel (`kernel.elf`) targeting the custom `x86_64-kazah_os.json` specification.
4. Setting up the Limine UEFI bootable tree in `iso_root/`.

Run the build script from the root of the repository:

```powershell
.\build.ps1
```

If the build succeeds, you will see:
```text
Build complete! The iso_root directory is ready.
```

---

## 🖥️ Running in QEMU

You can run the generated UEFI boot directory directly in QEMU:

```powershell
qemu-system-x86_64 `
    -m 512M `
    -bios "C:\Program Files\qemu\OVMF.fd" `
    -drive format=raw,file=fat:rw:iso_root `
    -serial stdio `
    -display none
```

*Note: Adjust the path to `OVMF.fd` according to your local QEMU installation.*

### Debugging with Serial Output
The kernel automatically redirects early boot logs, panic traces, and userspace diagnostics to the serial COM1 port (`0x3F8`), which is captured by `-serial stdio`.

---

## 🧪 Running Verification and Stress Suites

The system ships with comprehensive stress tests built into the userspace image:

- `stress_vfs`: Tests filesystem operations (`open`, `read`, `write`, `close`, `mkdir`) across `TmpFS` and `DevFS`.
- `stress_security`: Validates Mandatory Access Control (Biba Integrity Levels) and DAC Access Control Lists.
- `stress_ipc`: Saturates KazIPC channels with high-frequency concurrent messages.
- `stress_volume_manager`: Verifies volume creation, partition mapping, and virtual disk management.
- `stress_driver_crash`: Simulates hardware driver panics to verify supervisor self-healing and zero kernel downtime.

These tests are spawned automatically during the boot sequence by `service_manager` (PID 1) during integration runs.
