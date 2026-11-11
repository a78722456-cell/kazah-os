[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$ErrorActionPreference = "Stop"

$limineUrl = "https://github.com/limine-bootloader/limine/raw/v7.x-branch-binary/BOOTX64.EFI"

Write-Host "Building KAZAH OS Userspace..."
cargo build -p service_manager -p test_service -p crash_service -p stress_spawn -p stress_crash -p stress_ipc -p stress_mem -p stress_kill -p device_manager -p stress_pci -p stress_device_snap -p stress_registry -p stress_churn -p driver_manager -p ahci -p stress_driver_crash -p stress_driver_rebind -p ramdisk -p stress_storage_flood -p stress_storage_reset -p stress_storage_concurrent -p volume_manager -p stress_volume_manager -p stress_security -p stress_vfs -Z build-std=core,compiler_builtins,alloc -Z build-std-features=compiler-builtins-mem -Z json-target-spec --target x86_64-kazah_os.json
if ($LASTEXITCODE -ne 0) {
    Write-Host "Userspace build failed!"
    exit $LASTEXITCODE
}

if (!(Test-Path "build")) {
    New-Item -ItemType Directory -Force -Path "build" | Out-Null
}
Write-Host "Creating initrd.tar..."
tar -cf build/initrd.tar -C target/x86_64-kazah_os/debug/ service_manager test_service crash_service stress_spawn stress_crash stress_ipc stress_mem stress_kill device_manager stress_pci stress_device_snap stress_registry stress_churn driver_manager ahci stress_driver_crash stress_driver_rebind ramdisk stress_storage_flood stress_storage_reset stress_storage_concurrent volume_manager stress_volume_manager stress_security stress_vfs
if ($LASTEXITCODE -ne 0) {
    Write-Host "tar failed!"
    exit $LASTEXITCODE
}

Write-Host "Building KAZAH OS Kernel..."
cd kernel
cargo build -Z build-std=core,compiler_builtins,alloc -Z build-std-features=compiler-builtins-mem -Z json-target-spec --target ../x86_64-kazah_os.json
if ($LASTEXITCODE -ne 0) {
    Write-Host "Kernel build failed!"
    exit $LASTEXITCODE
}
cd ..

Write-Host "Creating ISO root directory..."
$isoRoot = "iso_root"
if (Test-Path $isoRoot) {
    Remove-Item -Recurse -Force $isoRoot
}
New-Item -ItemType Directory -Path "$isoRoot/EFI/BOOT" | Out-Null

Write-Host "Downloading Limine BOOTX64.EFI using curl..."
curl.exe -L -o "$isoRoot/EFI/BOOT/BOOTX64.EFI" $limineUrl

Write-Host "Copying kernel and config..."
Copy-Item "limine.conf" -Destination "$isoRoot/limine.conf"
Copy-Item "target/x86_64-kazah_os/debug/kernel" -Destination "$isoRoot/kernel.elf"

Write-Host "Build complete! The iso_root directory is ready."
