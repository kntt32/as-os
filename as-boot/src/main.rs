#![no_main]
#![no_std]

#[allow(unused)]
mod efi;
mod utf16;

use core::ops::Drop;
use core::{ffi::c_void, mem, panic::PanicInfo, ptr};
use core::slice;
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
        if let Err(msg) = load_as_kernel(system_table) {
            EfiSimpleTextOutputProtocol::output_string(con_out, "ERROR: ");
            EfiSimpleTextOutputProtocol::output_string(con_out, msg);
            EfiSimpleTextOutputProtocol::output_string(con_out, "\n\r");
            panic!();
        }

        EfiSimpleTextOutputProtocol::output_string(con_out, ".....\n\r");
    }

    loop {}
}

unsafe fn get_efi_simple_file_system_protocol(
    system_table: *const EfiSystemTable,
) -> Result<*const EfiSimpleFileSystemProtocol, &'static str> {
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
            return Err("simple file system protocol was not found in device handle");
        }

        Ok(simple_file_system_protocol)
    }
}

unsafe fn get_root_file_protocol(
    system_table: *const EfiSystemTable,
) -> Result<*const EfiFileProtocol, &'static str> {
    unsafe {
        let simple_file_system_protocol = get_efi_simple_file_system_protocol(system_table)?;
        let open_volume = (*simple_file_system_protocol).open_volume;

        let mut efi_file_protocol_root: *const EfiFileProtocol = ptr::null();
        if (open_volume)(
            simple_file_system_protocol,
            &mut efi_file_protocol_root as *mut _,
        ) != EFI_STATUS_SUCCESS
        {
            return Err("failed to get file system protocol from EfiSimpleFileSystemProtocol");
        }

        Ok(efi_file_protocol_root)
    }
}

unsafe fn read_kernel_file(system_table: *const EfiSystemTable) -> Result<FileBuff, &'static str> {
    // free_pages in caller
    unsafe {
        let efi_file_protocol_root = get_root_file_protocol(system_table)?;

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
            return Err("failed to get file system protocol from root directory");
        }

        let file_buff = FileBuff::new(system_table, efi_file_protocol_kernel);

        let close = (*efi_file_protocol_kernel).close;
        (close)(efi_file_protocol_kernel);

        let close = (*efi_file_protocol_root).close;
        (close)(efi_file_protocol_root);

        file_buff
    }
}

#[derive(Clone, Debug)]
pub struct FileBuff {
    pub system_table: *const EfiSystemTable,
    pub buff: *const u8,
    pub size: usize,
}

impl FileBuff {
    pub unsafe fn new(
        system_table: *const EfiSystemTable,
        file_handle: *const EfiFileProtocol,
    ) -> Result<FileBuff, &'static str> {
        unsafe {
            let file_info = Self::get_file_info(file_handle)?;
            let file_size = file_info.physical_size;

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
                return Err("failed to allocate tempolary buffer");
            }

            let buffer: *mut u8 = mem::transmute(allocated_page_address);
            let mut buffer_size: UIntN = allocate_page_len * 4096;

            let read = (*file_handle).read;
            if (read)(file_handle, &mut buffer_size as *mut UIntN, buffer) != EFI_STATUS_SUCCESS {
                return Err("failed to read file to buffer");
            }

            Ok(FileBuff {
                system_table: system_table,
                buff: buffer,
                size: file_size as usize,
            })
        }
    }

    unsafe fn get_file_info(
        file_handle: *const EfiFileProtocol,
    ) -> Result<EfiFileInfo, &'static str> {
        unsafe {
            let file_info_guid = EfiFileInfo::GUID;
            let mut file_info_buff = [0u8; 1024];
            let mut file_info_size: UIntN = file_info_buff.len();

            if ((*file_handle).get_info)(
                file_handle,
                &file_info_guid,
                &mut file_info_size as *mut _,
                &mut file_info_buff as *mut u8,
            ) != EFI_STATUS_SUCCESS
            {
                return Err("failed to get EfiFileInfo");
            }

            let mut file_info: EfiFileInfo = mem::zeroed();
            (&file_info_buff as *const u8).copy_to(
                &mut file_info as *mut _ as *mut u8,
                size_of::<EfiFileInfo>(),
            );

            Ok(file_info)
        }
    }
}

impl Drop for FileBuff {
    fn drop(&mut self) {
        unsafe {
            let boot_services = (*self.system_table).boot_services;
            let free_pages = (*boot_services).free_pages;

            (free_pages)(self.buff as EfiPhysicalAddress, (self.size + 4095) / 4096);
        }
    }
}

unsafe fn load_as_kernel(system_table: *const EfiSystemTable) -> Result<*const u8, &'static str> {
    unsafe {
        let kernel_file_buffer = read_kernel_file(system_table)?;
        let kernel_file_buff_u8_slice = slice::from_raw_parts(kernel_file_buffer.buff, kernel_file_buffer.size);
        let Some(kernel_elf) = Elf64::new(kernel_file_buff_u8_slice) else {
            return Err("invalid elf file");
        };

        let Some(expand_info) = kernel_elf.expand_info() else {
            return Err("failed to get kernel expand size");
        };

        Err("TODO!")
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
