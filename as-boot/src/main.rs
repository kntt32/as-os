#![no_main]
#![no_std]

#[allow(unused)]
mod efi;
mod utf16;

use core::{ffi::c_void, mem, panic::PanicInfo, ptr};
use efi::{
    EFI_STATUS_SUCCESS, EfiAllocateType, EfiFileInfo, EfiFileProtocol, EfiHandle, EfiMemoryType,
    EfiPhysicalAddress, EfiSimpleFileSystemProtocol, EfiSimpleTextOutputProtocol, EfiStatus,
    EfiSystemTable, UIntN,
};
use elf::Elf64;

#[unsafe(no_mangle)]
pub unsafe extern "efiapi" fn efi_main(
    _image_handle: EfiHandle,
    system_table: *const EfiSystemTable,
) -> EfiStatus {
    unsafe {
        let con_out = system_table
            .as_ref()
            .expect("expected non-null pointer")
            .con_out;
        EfiSimpleTextOutputProtocol::clear_screen(con_out);
        EfiSimpleTextOutputProtocol::output_string(con_out, "Hello, World!\n\r");

        EfiSimpleTextOutputProtocol::output_string(con_out, "LOG: loading as-kernel ...\n\r");
        load_as_kernel(system_table);

        EfiSimpleTextOutputProtocol::output_string(con_out, ".....\n\r");
    }

    loop {}
}

unsafe fn get_efi_simple_file_system_protocol(
    system_table: *const EfiSystemTable,
    std_err: *const EfiSimpleTextOutputProtocol,
) -> *const EfiSimpleFileSystemProtocol {
    unsafe {
        let boot_services = (*system_table).boot_services;
        let locate_protocol = (*boot_services).locate_protocol;

        let simple_file_system_protocol_guid = EfiSimpleFileSystemProtocol::GUID;
        let mut simple_file_system_protocol: *const EfiSimpleFileSystemProtocol = ptr::null();
        if (locate_protocol)(
            &simple_file_system_protocol_guid,
            ptr::null(),
            &mut simple_file_system_protocol as *mut *const _ as *mut *const c_void,
        ) != EFI_STATUS_SUCCESS
        {
            EfiSimpleTextOutputProtocol::output_string(
                std_err,
                "ERROR: simple file system protocol was not found in device handle\n\r",
            );
            panic!("ERROR: simple file system protocol was not found in device handle\n\r");
        }

        simple_file_system_protocol
    }
}

unsafe fn get_root_file_protocol(
    system_table: *const EfiSystemTable,
    std_err: *const EfiSimpleTextOutputProtocol,
) -> *const EfiFileProtocol {
    unsafe {
        let simple_file_system_protocol =
            get_efi_simple_file_system_protocol(system_table, std_err);
        let open_volume = (*simple_file_system_protocol).open_volume;

        let mut efi_file_protocol_root: *const EfiFileProtocol = ptr::null();
        if (open_volume)(
            simple_file_system_protocol,
            &mut efi_file_protocol_root as *mut _,
        ) != EFI_STATUS_SUCCESS
        {
            EfiSimpleTextOutputProtocol::output_string(
                std_err,
                "ERROR: failed to get file system protocol from EfiSimpleFileSystemProtocol\n\r",
            );
            panic!(
                "ERROR: failed to get file system protocol from EfiSimpleFileSystemProtocol\n\r"
            );
        }

        efi_file_protocol_root
    }
}

unsafe fn read_kernel_file(
    system_table: *const EfiSystemTable,
    std_err: *const EfiSimpleTextOutputProtocol,
) -> *const u8 {
    // free_pages in caller
    unsafe {
        let efi_file_protocol_root = get_root_file_protocol(system_table, std_err);

        let open = (*efi_file_protocol_root).open;

        let kernel_path_utf16 = utf16::as_utf16::<16>("kernel.elf");
        let mut efi_file_protocol_kernel: *const EfiFileProtocol = ptr::null();
        if (open)(
            efi_file_protocol_root,
            &mut efi_file_protocol_kernel as *mut _,
            &kernel_path_utf16 as *const _,
            EfiFileProtocol::EFI_FILE_MODE_READ,
            0,
        ) != EFI_STATUS_SUCCESS
        {
            EfiSimpleTextOutputProtocol::output_string(
                std_err,
                "ERROR: failed to get file system protocol from root directory\n\r",
            );
            panic!("ERROR: failed to get file system protocol from root directory\n\r");
        }

        let buffer: *const u8 = read_to_memory(system_table, std_err, efi_file_protocol_kernel);

        let close = (*efi_file_protocol_kernel).close;
        (close)(efi_file_protocol_kernel);

        let close = (*efi_file_protocol_root).close;
        (close)(efi_file_protocol_root);

        buffer
    }
}

unsafe fn read_to_memory(
    system_table: *const EfiSystemTable,
    std_err: *const EfiSimpleTextOutputProtocol,
    file_handle: *const EfiFileProtocol,
) -> *const u8 {
    // free_pages in caller
    let file_info_guid = EfiFileInfo::GUID;
    let mut file_info_buff = [0u8; 1024];
    let mut file_info_size: UIntN = file_info_buff.len();

    unsafe {
        if ((*file_handle).get_info)(
            file_handle,
            &file_info_guid,
            &mut file_info_size as *mut _,
            &mut file_info_buff as *mut u8,
        ) != EFI_STATUS_SUCCESS
        {
            EfiSimpleTextOutputProtocol::output_string(
                std_err,
                "ERROR: failed to get EfiFileInfo\n\r",
            );
            panic!("ERROR: failed to get EfiFileInfo\n\r");
        }

        let file_info_ptr =
            mem::transmute::<*const u8, *const EfiFileInfo>(&file_info_buff as *const u8);
        let file_size = (*file_info_ptr).physical_size;

        let boot_services = (*system_table).boot_services;
        let allocate_pages = (*boot_services).allocate_pages;
        let allocate_page_len: UIntN = (file_size as UIntN + 4095) / 4096;
        let mut allocated_page_address: EfiPhysicalAddress = 0;
        if (allocate_pages)(
            EfiAllocateType::AllocateAnyPages,
            EfiMemoryType::EfiLoaderData,
            allocate_page_len,
            &mut allocated_page_address as *mut _,
        ) != EFI_STATUS_SUCCESS
        {
            EfiSimpleTextOutputProtocol::output_string(
                std_err,
                "ERROR: failed to allocate tempolary buffer\n\r",
            );
            panic!("ERROR: failed to allocate tempolary buffer\n\r");
        }

        let buffer: *mut u8 = mem::transmute(allocated_page_address);
        let mut buffer_size: UIntN = allocate_page_len * 4096;

        let read = (*file_handle).read;
        if (read)(file_handle, &mut buffer_size as *mut UIntN, buffer) != EFI_STATUS_SUCCESS {
            EfiSimpleTextOutputProtocol::output_string(
                std_err,
                "ERROR: failed to read file to buffer\n\r",
            );
            panic!("ERROR: failed to read file to buffer\n\r");
        }

        buffer
    }
}

unsafe fn load_as_kernel(system_table: *const EfiSystemTable) {
    unsafe {
        let std_err = system_table
            .as_ref()
            .expect("expected non-null pointer")
            .con_out;

        let _kernel_temp_buffer: *const u8 = read_kernel_file(system_table, std_err);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
