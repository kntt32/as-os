use fatfs::{FatType, FileSystem, FormatVolumeOptions, FsOptions};
use std::{
    env,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
    process::Command,
};

fn main() {
    let args: Vec<_> = env::args().collect();
    let release_flag = 2 <= args.len() && args[1] == "release";

    build_as_boot(release_flag);
    build_as_kernel(release_flag);
    create_disk(release_flag);
    run_qemu();
}

fn build_as_boot(release_flag: bool) {
    println!("building as-boot ...");
    env::set_current_dir("as-boot").unwrap();
    let mut command = Command::new("cargo");
    command.arg("build");
    if release_flag {
        command.arg("--release");
    }
    if !command.status().unwrap().success() {
        panic!("failed to build as-boot");
    }
    env::set_current_dir("../").unwrap();
}

fn build_as_kernel(release_flag: bool) {
    println!("building as-kernel ...");
    env::set_current_dir("as-kernel").unwrap();
    let mut command = Command::new("cargo");
    command.arg("build");
    if release_flag {
        command.arg("--release");
    }
    if !command.status().unwrap().success() {
        panic!("failed to build as-boot");
    }
    env::set_current_dir("../").unwrap();
}

fn create_disk(release_flag: bool) {
    println!("creating disk image ...");

    let as_boot_path = if release_flag {
        Path::new("target/x86_64-unknown-uefi/release/as-boot.efi")
    } else {
        Path::new("target/x86_64-unknown-uefi/debug/as-boot.efi")
    };
    let mut as_boot = File::open(as_boot_path).unwrap();
    let mut as_boot_vec = Vec::new();
    as_boot.read_to_end(&mut as_boot_vec).unwrap();

    let as_kernel_path = if release_flag {
        Path::new("target/x86_64-unknown-none/release/as-kernel")
    } else {
        Path::new("target/x86_64-unknown-none/debug/as-kernel")
    };
    let mut as_kernel = File::open(as_kernel_path).unwrap();
    let mut as_kernel_vec = Vec::new();
    as_kernel.read_to_end(&mut as_kernel_vec).unwrap();

    let disk_path = Path::new("target/disk.img");
    let mut disk = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(disk_path)
        .unwrap();
    disk.set_len(1024 * 1024 * 64).unwrap();
    fatfs::format_volume(
        &mut disk,
        FormatVolumeOptions::new().fat_type(FatType::Fat32),
    )
    .unwrap();

    let file_system = FileSystem::new(&mut disk, FsOptions::new()).unwrap();
    let file_system_root = file_system.root_dir();
    let file_system_efi_boot = file_system_root
        .create_dir("EFI")
        .unwrap()
        .create_dir("BOOT")
        .unwrap();

    let mut bootx64 = file_system_efi_boot.create_file("BOOTX64.EFI").unwrap();
    bootx64.write_all(&as_boot_vec).unwrap();

    let mut kernel = file_system_root.create_file("kernel.elf").unwrap();
    kernel.write_all(&as_kernel_vec).unwrap();
}

fn run_qemu() {
    println!("running qemu ...");

    Command::new("qemu-system-x86_64")
        .args([
            "-bios",
            "ovmf/OVMF.fd",
            "-drive",
            "format=raw,file=target/disk.img",
            "-m",
            "512M",
            "--enable-kvm",
        ])
        .status()
        .unwrap();
}
