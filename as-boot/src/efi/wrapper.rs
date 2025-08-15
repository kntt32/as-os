#[macro_use]
pub mod static_str;

use super::*;

use core::ffi::c_void;
use core::fmt;
use core::fmt::Write;
use core::mem;
use core::mem::MaybeUninit;
use core::ops::Deref;
use core::ops::DerefMut;
use core::panic::PanicInfo;
use core::ptr;
use core::slice;
use static_str::*;

static mut SYSTEM_TABLE: *const EfiSystemTable = ptr::null();
static mut BOOT_SERVICES: *const EfiBootServices = ptr::null();

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if check_boot_services_is_avaiable().is_ok() {
        if let Some(location) = info.location() {
            println!(
                "paniced at {}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            );
        } else {
            println!("paniced");
        }

        if let Some(s) = info.message().as_str() {
            stdout(s);
        }
    }
    loop {}
}

pub unsafe fn init(system_table: *const EfiSystemTable) {
    unsafe {
        SYSTEM_TABLE = system_table;
        if SYSTEM_TABLE != ptr::null() {
            BOOT_SERVICES = (&*system_table).boot_services;
        }
    }
}

fn check_boot_services_is_avaiable() -> Result<(), &'static str> {
    if unsafe { SYSTEM_TABLE == ptr::null() || BOOT_SERVICES == ptr::null() } {
        Err("unexpected call of stdout")
    } else {
        Ok(())
    }
}

pub fn stdout(string: &str) -> Result<(), &'static str> {
    check_boot_services_is_avaiable()?;

    let con_out = unsafe { (&*SYSTEM_TABLE).con_out };
    let output_string = unsafe { (&*con_out).output_string };

    let mut string_encode_utf16 = string.encode_utf16();
    loop {
        let mut string_utf16_buff = [0u16; 1024];
        let mut string_utf16_len: usize = 0;

        while string_utf16_len < string_utf16_buff.len() - 1 {
            let Some(utf16_code) = string_encode_utf16.next() else {
                break;
            };

            string_utf16_buff[string_utf16_len] = utf16_code;
            string_utf16_len += 1;
        }
        string_utf16_buff[string_utf16_buff.len() - 1] = 0;

        if string_utf16_len == 0 {
            break Ok(());
        }

        let status = unsafe { (output_string)(con_out, &raw const string_utf16_buff as *const _) };
        if status != EFI_STATUS_SUCCESS {
            panic!("failed to output string");
        }
    }
}

pub fn stdclean() -> Result<(), &'static str> {
    check_boot_services_is_avaiable()?;

    unsafe {
        let con_out = (&*SYSTEM_TABLE).con_out;
        let clear_screen = (&*con_out).clear_screen;

        if (clear_screen)(con_out) == EFI_STATUS_SUCCESS {
            Ok(())
        } else {
            Err("failed to clear screen")
        }
    }
}

pub fn alloc_pages(pages: usize) -> *mut u8 {
    check_boot_services_is_avaiable().expect("use after exit_boot_services");

    if pages == 0 {
        ptr::null_mut()
    } else {
        let allocate_pages = unsafe { (&*BOOT_SERVICES).allocate_pages };
        let mut memory: EfiPhysicalAddress = 0;

        if unsafe {
            (allocate_pages)(
                EfiAllocateType::AllocateAnyPages,
                EfiMemoryType::EfiLoaderData,
                pages,
                &raw mut memory,
            )
        } != EFI_STATUS_SUCCESS
        {
            panic!("failed to alloc pages");
        }

        let ptr: *mut u8 = memory as usize as *mut u8;
        ptr
    }
}

pub fn dealloc_pages(ptr: *mut u8, pages: usize) {
    check_boot_services_is_avaiable().expect("use after exit_boot_services");

    if pages != 0 {
        let free_pages = unsafe { (&*BOOT_SERVICES).free_pages };
        let memory: EfiPhysicalAddress = ptr.addr() as EfiPhysicalAddress;

        if unsafe { (free_pages)(memory, pages) } != EFI_STATUS_SUCCESS {
            panic!("failed to free pages");
        }
    }
}

#[derive(Debug)]
pub struct PageBox {
    page: *mut u8,
    pages: usize,
}

impl PageBox {
    pub const PAGE_SIZE: usize = 4096;

    pub fn new(pages: usize) -> Self {
        let page: *mut u8 = alloc_pages(pages);

        Self {
            page: page,
            pages: pages,
        }
    }

    pub fn new_from_bytes(size: usize) -> Self {
        let alloc_pages = (size + Self::PAGE_SIZE - 1) / Self::PAGE_SIZE;
        Self::new(alloc_pages)
    }

    pub fn leak(self) -> &'static mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.page, self.pages * Self::PAGE_SIZE) }
    }
}

impl Drop for PageBox {
    fn drop(&mut self) {
        dealloc_pages(self.page, self.pages);
    }
}

impl Deref for PageBox {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.page, self.pages * Self::PAGE_SIZE) }
    }
}

impl DerefMut for PageBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.page, self.pages * Self::PAGE_SIZE) }
    }
}

#[derive(Clone, Debug)]
pub struct File {
    protocol: *const EfiFileProtocol,
}

impl Drop for File {
    fn drop(&mut self) {
        if check_boot_services_is_avaiable().is_ok() {
            unsafe {
                let close = (&*self.protocol).close;
                (close)(self.protocol);
            }
        }
    }
}

impl File {
    pub fn new(path: &str) -> Result<Self, &'static str> {
        check_boot_services_is_avaiable()?;
        let root = Self::get_root()?;
        let open = unsafe { (&*root).open };

        let path_utf16 = utf16::as_utf16::<1024>(path);

        let mut protocol = ptr::null();
        unsafe {
            if (open)(
                root,
                &raw mut protocol,
                &raw const path_utf16 as *const _,
                EfiFileProtocol::EFI_FILE_MODE_READ,
                0,
            ) != EFI_STATUS_SUCCESS
            {
                return Err("failed to open file");
            }
        }

        Ok(Self { protocol: protocol })
    }

    pub fn size(&self) -> usize {
        check_boot_services_is_avaiable().expect("use after exit_boot_services");

        let get_info = unsafe { (&*self.protocol).get_info };

        let file_info_guid = EfiFileInfo::GUID;
        let mut file_info_buff = [0u8; 1024];
        let mut file_info_buff_size: UIntN = file_info_buff.len();

        if unsafe {
            (get_info)(
                self.protocol,
                &raw const file_info_guid,
                &raw mut file_info_buff_size,
                &raw mut file_info_buff as _,
            )
        } != EFI_STATUS_SUCCESS
        {
            panic!("failed to get file size");
        }

        let mut file_info_maybe_uninit = MaybeUninit::<EfiFileInfo>::uninit();
        let file_info;
        unsafe {
            (&raw const file_info_buff as *const u8).copy_to(
                file_info_maybe_uninit.as_mut_ptr() as *mut u8,
                mem::size_of::<EfiFileInfo>(),
            );
            file_info = file_info_maybe_uninit.assume_init();
        }

        file_info.file_size as usize
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, &'static str> {
        check_boot_services_is_avaiable()?;

        let read = unsafe { (&*self.protocol).read };

        let mut buffer_size: UIntN = buf.len();
        let mut buffer: *mut u8 = buf as *mut _ as *mut u8;
        if unsafe { (read)(self.protocol, &raw mut buffer_size, buffer) } != EFI_STATUS_SUCCESS {
            return Err("failed to read to buffer");
        }

        Ok(buffer_size)
    }

    fn get_root() -> Result<*const EfiFileProtocol, &'static str> {
        static mut ROOT: *const EfiFileProtocol = ptr::null();

        if unsafe { ROOT.is_null() } {
            let volume = Self::get_volume()?;
            unsafe {
                let open_volume = (&*volume).open_volume;
                if (open_volume)(volume, &raw mut ROOT) != EFI_STATUS_SUCCESS {
                    ROOT = ptr::null();
                    return Err("failed to open volume");
                }
            }
        }

        unsafe { Ok(ROOT) }
    }

    fn get_volume() -> Result<*const EfiSimpleFileSystemProtocol, &'static str> {
        check_boot_services_is_avaiable()?;

        let locate_protocol = unsafe { (&*BOOT_SERVICES).locate_protocol };

        let efi_simple_file_system_protocol_guid = EfiSimpleFileSystemProtocol::GUID;
        let mut efi_simple_file_system_protocol: *const EfiSimpleFileSystemProtocol =
            ptr::null_mut();
        if unsafe {
            (locate_protocol)(
                &raw const efi_simple_file_system_protocol_guid,
                ptr::null(),
                &raw mut efi_simple_file_system_protocol as *mut *const c_void,
            )
        } == EFI_STATUS_SUCCESS
        {
            Ok(efi_simple_file_system_protocol)
        } else {
            Err("failed to locate EfiSimpleFileSystemProtocol")
        }
    }
}
