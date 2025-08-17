#![no_std]

pub mod utf16;

use core::ffi::c_void;

pub type Boolean = bool;
pub type IntN = isize;
pub type UIntN = usize;
pub type Int8 = i8;
pub type UInt8 = u8;
pub type Int16 = i16;
pub type UInt16 = u16;
pub type Int32 = i32;
pub type UInt32 = u32;
pub type Int64 = i64;
pub type UInt64 = u64;
pub type Int128 = i128;
pub type UInt128 = u128;
pub type Char8 = u8;
pub type Char16 = u16;
pub type EfiStatus = UIntN;
pub type EfiHandle = *mut c_void;
pub type EfiEvent = *mut c_void;
pub type EfiLba = UInt64;
pub type EfiTpl = UIntN;

pub const EFI_STATUS_SUCCESS: EfiStatus = 0;
pub const EFI_STATUS_ERROR: EfiStatus = EfiStatus::MAX;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EfiGuid(pub UInt32, pub UInt16, pub UInt16, pub [UInt8; 8]);

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfiTableHeader {
    signature: UInt64,
    revision: UInt32,
    header_size: UInt32,
    crc32: UInt32,
    reserved: UInt32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfiSystemTable {
    pub hdr: EfiTableHeader,
    pub firmware_vendor: *const Char16,
    pub firmware_revision: UInt32,
    pub console_in_handle: EfiHandle,
    pub con_in: *const EfiSimpleTextInputProtocol,
    pub console_out_handle: EfiHandle,
    pub con_out: *const EfiSimpleTextOutputProtocol,
    standard_error_handle: EfiHandle,
    pub std_err: *const EfiSimpleTextOutputProtocol,
    pub runtime_services: *const EfiRuntimeServices,
    pub boot_services: *const EfiBootServices,
    pub number_of_table_entries: UIntN,
    pub configuration_table: *const EfiConfigurationTable,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfiConfigurationTable {
    pub vendor_guid: EfiGuid,
    pub vendor_table: *const c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfiInputKey {
    scan_code: UInt16,
    unicode_char: Char16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfiSimpleTextInputProtocol {
    reset: unsafe extern "efiapi" fn(
        this: *const EfiSimpleTextInputProtocol,
        extended_verification: Boolean,
    ) -> EfiStatus,
    read_key_stroke: unsafe extern "efiapi" fn(
        this: *const EfiSimpleTextInputProtocol,
        key: *mut EfiInputKey,
    ) -> EfiStatus,
    wait_for_key: EfiEvent,
}

impl EfiSimpleTextInputProtocol {
    pub const GUID: EfiGuid = EfiGuid(
        0x387477c1,
        0x69c7,
        0x11d2,
        [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
    );
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfiSimpleTextOutputProtocol {
    reset: unsafe extern "efiapi" fn(
        this: *const EfiSimpleTextOutputProtocol,
        extended_verification: Boolean,
    ) -> EfiStatus,
    pub output_string: unsafe extern "efiapi" fn(
        this: *const EfiSimpleTextOutputProtocol,
        string: *const Char16,
    ) -> EfiStatus,
    test_string: unsafe extern "efiapi" fn(
        this: *const EfiSimpleTextOutputProtocol,
        string: *const Char16,
    ) -> EfiStatus,
    query_mode: unsafe extern "efiapi" fn(
        this: *const EfiSimpleTextOutputProtocol,
        mode_number: UIntN,
        columns: *mut UIntN,
        rows: *mut UIntN,
    ) -> EfiStatus,
    set_mode: unsafe extern "efiapi" fn(
        this: *const EfiSimpleTextOutputProtocol,
        mode_number: UIntN,
    ) -> EfiStatus,
    set_attribute: unsafe extern "efiapi" fn(
        this: *const EfiSimpleTextOutputProtocol,
        attribute: UIntN,
    ) -> EfiStatus,
    pub clear_screen:
        unsafe extern "efiapi" fn(this: *const EfiSimpleTextOutputProtocol) -> EfiStatus,
    set_cursor_position: unsafe extern "efiapi" fn(
        this: *const EfiSimpleTextOutputProtocol,
        column: UIntN,
        row: UIntN,
    ) -> EfiStatus,
    enable_cursor: unsafe extern "efiapi" fn(
        this: *const EfiSimpleTextOutputProtocol,
        visible: Boolean,
    ) -> EfiStatus,
    mode: *const SimpleTextOutputMode,
}

impl EfiSimpleTextOutputProtocol {
    pub const GUID: EfiGuid = EfiGuid(
        0x387477c2,
        0x69c7,
        0x11d2,
        [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
    );

    pub unsafe fn clear_screen(this: *const Self) -> EfiStatus {
        unsafe {
            (this
                .as_ref()
                .expect("expected non-null pointer")
                .clear_screen)(this)
        }
    }

    pub unsafe fn output_string(this: *const Self, string: &str) -> EfiStatus {
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
                break EFI_STATUS_SUCCESS;
            }

            let status = unsafe {
                (this
                    .as_ref()
                    .expect("expected non-null pointer")
                    .output_string)(this, &string_utf16_buff as *const u16)
            };
            if status != EFI_STATUS_SUCCESS {
                break status;
            }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SimpleTextOutputMode {
    max_mode: Int32,
    mode: Int32,
    attribute: Int32,
    cursor_column: Int32,
    cursor_row: Int32,
    cursor_visible: Boolean,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfiRuntimeServices {
    hdr: EfiTableHeader,

    get_time: *const c_void,
    set_time: *const c_void,
    get_wakeup_time: *const c_void,
    set_wakeup_time: *const c_void,

    set_virtual_address_map: unsafe extern "efiapi" fn(
        memory_map_size: UIntN,
        descriptor_size: UIntN,
        descriptor_version: UIntN,
        virtual_map: *const EfiMemoryDescriptor,
    ) -> EfiStatus,
    convert_pointer:
        unsafe extern "efiapi" fn(debug_disposition: UIntN, address: *mut *mut c_void) -> EfiStatus,

    get_variable: unsafe extern "efiapi" fn(
        variable_name: *const Char16,
        vendor_guid: *const EfiGuid,
        attributes: *mut UInt32,
        data_size: *mut UIntN,
        data: *mut c_void,
    ) -> EfiStatus,
    get_next_variable_name: unsafe extern "efiapi" fn(
        variable_name_size: *mut UIntN,
        variable_name: *mut Char16,
        vendor_guid: *mut EfiGuid,
    ) -> EfiStatus,
    set_variable: unsafe extern "efiapi" fn(
        variable_name: *const Char16,
        vendor_guid: *const EfiGuid,
        attributes: *const UInt32,
        data_size: UIntN,
        data: *const c_void,
    ) -> EfiStatus,

    get_next_high_monotonic_count: *const c_void,
    reset_system: unsafe extern "efiapi" fn(
        reset_type: EfiResetType,
        reset_status: EfiStatus,
        data_size: UIntN,
        reset_data: *const c_void,
    ) -> !,
    update_capsule: *const c_void,
    query_capsule_capabilities: *const c_void,
    query_variable_info: *const c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub enum EfiResetType {
    EfiResetCold = 0,
    EfiResetWarm = 1,
    EfiResetShutdown = 2,
    EfiResetPlatformSpecific = 3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfiBootServices {
    hdr: EfiTableHeader,

    raise_tpl: *const c_void,
    restore_tpl: *const c_void,

    pub allocate_pages: unsafe extern "efiapi" fn(
        r#type: EfiAllocateType,
        memory_type: EfiMemoryType,
        pages: UIntN,
        memory: *mut EfiPhysicalAddress,
    ) -> EfiStatus,
    pub free_pages:
        unsafe extern "efiapi" fn(memory: EfiPhysicalAddress, pages: UIntN) -> EfiStatus,
    pub get_memory_map: unsafe extern "efiapi" fn(
        memory_map_size: *mut UIntN,
        memory_map: *mut EfiMemoryDescriptor,
        map_key: *mut UIntN,
        descriptor_size: *mut UIntN,
        descriptor_version: *mut UInt32,
    ) -> EfiStatus,
    allocate_pool: *const c_void,
    free_pool: *const c_void,

    create_event: *const c_void,
    set_timer: *const c_void,
    wait_for_event: *const c_void,
    signal_event: *const c_void,
    close_event: *const c_void,
    check_event: *const c_void,

    install_protocol_interface: *const c_void,
    reinstall_protocol_interface: *const c_void,
    uninstall_protocol_interface: *const c_void,
    pub handle_protocol: unsafe extern "efiapi" fn(
        handle: EfiHandle,
        protocol: *const EfiGuid,
        interface: *mut *const c_void,
    ) -> EfiStatus,
    _reserved: *const c_void,
    register_protocol_notify: *const c_void,
    locate_handle: unsafe extern "efiapi" fn(
        search_type: EfiLocateSearchType,
        protocol: *const EfiGuid,
        search_key: *const c_void,
        buffer_size: *mut UIntN,
        buffer: *mut EfiHandle,
    ) -> EfiStatus,
    locate_device_path: *const c_void,
    install_configurtaion_table: *const c_void,

    load_image: *const c_void,
    start_image: *const c_void,
    exit: *const c_void,
    unload_image: *const c_void,
    exit_boot_services:
        unsafe extern "efiapi" fn(image_handle: EfiHandle, map_key: UIntN) -> EfiStatus,

    get_next_monotonic_count: *const c_void,
    stall: *const c_void,
    set_watchdog_timer: *const c_void,

    connect_controller: *const c_void,
    disconnect_controller: *const c_void,

    open_protocol: unsafe extern "efiapi" fn(
        handle: EfiHandle,
        protocol: *const EfiGuid,
        interface: *mut *const c_void,
        agent_handle: EfiHandle,
        controller_handle: EfiHandle,
        attributes: UInt32,
    ) -> EfiStatus,
    close_protocol: unsafe extern "efiapi" fn(
        handle: EfiHandle,
        protocol: *const EfiGuid,
        agent_handle: EfiHandle,
        controller_handle: *const EfiHandle,
    ) -> EfiStatus,
    open_protocol_information: *const c_void,

    protocols_per_handle: *const c_void,
    locate_handle_buffer: *const c_void,
    pub locate_protocol: unsafe extern "efiapi" fn(
        protocol: *const EfiGuid,
        registration: *const c_void,
        interface: *mut *const c_void,
    ) -> EfiStatus,
    install_multiple_protocol_interfaces: *const c_void,
    uninstall_multiple_protocol_interfaces: *const c_void,

    calculate_crc32: *const c_void,

    copy_mem: *const c_void,
    set_mem: *const c_void,
    create_event_ex: *const c_void,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum EfiLocateSearchType {
    AllHandles = 0,
    ByRegisterNotify = 1,
    ByProtocol = 2,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum EfiAllocateType {
    AllocateAnyPages = 0,
    AllocateMaxAddress = 1,
    AllocateAddress = 2,
    MaxAllocateType = 3,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum EfiMemoryType {
    EfiReservedMemoryType = 0,
    EfiLoaderCode = 1,
    EfiLoaderData = 2,
    EfiBootServicesCode = 3,
    EfiBootServicesData = 4,
    EfiRuntimeServicesCode = 5,
    EfiRuntimeServicesData = 6,
    EfiConventionalMemory = 7,
    EfiUnusableMemory = 8,
    EfiACPIReclaimMemory = 9,
    EfiACPIMemoryNVS = 10,
    EfiMemoryMappedIO = 11,
    EfiMemoryMappedIOPortSpace = 12,
    EfiPalCode = 13,
    EfiPersistentMemory = 14,
    EfiUnacceptedMemoryType = 15,
    EfiMaxMemoryType = 16,
}

pub type EfiPhysicalAddress = UInt64;
pub type EfiVirtualAddress = UInt64;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfiMemoryDescriptor {
    r#type: EfiMemoryType,
    physical_start: EfiPhysicalAddress,
    virtual_start: EfiVirtualAddress,
    number_of_pages: UInt64,
    attribute: UInt64,
}

impl EfiMemoryDescriptor {
    pub const EFI_MEMORY_UC: UInt64 = 0x0000000000000001;
    pub const EFI_MEMORY_WC: UInt64 = 0x0000000000000002;
    pub const EFI_MEMORY_WT: UInt64 = 0x0000000000000004;
    pub const EFI_MEMORY_WB: UInt64 = 0x0000000000000008;
    pub const EFI_MEMORY_UCE: UInt64 = 0x0000000000000010;
    pub const EFI_MEMORY_WP: UInt64 = 0x0000000000001000;
    pub const EFI_MEMORY_RP: UInt64 = 0x0000000000002000;
    pub const EFI_MEMORY_XP: UInt64 = 0x0000000000004000;
    pub const EFI_MEMORY_NV: UInt64 = 0x0000000000008000;
    pub const EFI_MEMORY_MORE_RELIABLE: UInt64 = 0x0000000000010000;
    pub const EFI_MEMORY_RO: UInt64 = 0x0000000000020000;
    pub const EFI_MEMORY_SP: UInt64 = 0x0000000000040000;
    pub const EFI_MEMORY_CPU_CRYPTO: UInt64 = 0x0000000000080000;
    pub const EFI_MEMORY_RUNTIME: UInt64 = 0x8000000000000000;
    pub const EFI_MEMORY_ISA_VALID: UInt64 = 0x4000000000000000;
    pub const EFI_MEMORY_ISA_MASK: UInt64 = 0x0FFFF00000000000;
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfiSimpleFileSystemProtocol {
    revision: UInt64,
    pub open_volume: extern "efiapi" fn(
        this: *const EfiSimpleFileSystemProtocol,
        root: *mut *const EfiFileProtocol,
    ) -> EfiStatus,
}

impl EfiSimpleFileSystemProtocol {
    pub const GUID: EfiGuid = EfiGuid(
        0x0964e5b22,
        0x6459,
        0x11d2,
        [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
    );
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfiFileProtocol {
    revision: UInt64,
    pub open: unsafe extern "efiapi" fn(
        this: *const EfiFileProtocol,
        new_handle: *mut *const EfiFileProtocol,
        file_name: *const Char16,
        open_mode: UInt64,
        attributes: UInt64,
    ) -> EfiStatus,
    pub close: unsafe extern "efiapi" fn(this: *const EfiFileProtocol) -> EfiStatus,
    delete: unsafe extern "efiapi" fn(this: *const EfiFileProtocol) -> EfiStatus,
    pub read: unsafe extern "efiapi" fn(
        this: *const EfiFileProtocol,
        buffer_size: *mut UIntN,
        buffer: *mut u8,
    ) -> EfiStatus,
    write: unsafe extern "efiapi" fn(
        this: *const EfiFileProtocol,
        buffer_size: *mut UIntN,
        buffer: *const u8,
    ) -> EfiStatus,
    get_position: usize,
    set_position: usize,
    pub get_info: unsafe extern "efiapi" fn(
        this: *const EfiFileProtocol,
        information_type: *const EfiGuid,
        buffer_size: *mut UIntN,
        buffer: *mut u8,
    ) -> EfiStatus,
    set_info: usize,
    flush: usize,
    open_ex: usize,
    read_ex: usize,
    write_ex: usize,
    flush_ex: usize,
}

impl EfiFileProtocol {
    pub const EFI_FILE_MODE_READ: UInt64 = 0x0000000000000001;
    pub const EFI_FILE_MODE_WRITE: UInt64 = 0x0000000000000002;
    pub const EFI_FILE_MODE_CREATE: UInt64 = 0x8000000000000000;

    pub const EFI_FILE_READ_ONLY: UInt64 = 0x0000000000000001;
    pub const EFI_FILE_HIDDEN: UInt64 = 0x0000000000000002;
    pub const EFI_FILE_SYSTEM: UInt64 = 0x0000000000000004;
    pub const EFI_FILE_RESERVED: UInt64 = 0x0000000000000008;
    pub const EFI_FILE_DIRECTORY: UInt64 = 0x0000000000000010;
    pub const EFI_FILE_ARCHIVE: UInt64 = 0x0000000000000020;
    pub const EFI_FILE_VALID_ATTR: UInt64 = 0x0000000000000037;
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfiFileInfo {
    pub size: UInt64,
    pub file_size: UInt64,
    pub physical_size: UInt64,
    pub create_time: EfiTime,
    pub last_access_time: EfiTime,
    pub modification_time: EfiTime,
    pub attribute: UInt64,
}

impl EfiFileInfo {
    pub const GUID: EfiGuid = EfiGuid(
        0x09576e92,
        0x6d3f,
        0x11d2,
        [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
    );

    pub const EFI_FILE_READ_ONLY: UInt64 = 0x0000000000000001;
    pub const EFI_FILE_HIDDEN: UInt64 = 0x0000000000000002;
    pub const EFI_FILE_SYSTEM: UInt64 = 0x0000000000000004;
    pub const EFI_FILE_RESERVED: UInt64 = 0x0000000000000008;
    pub const EFI_FILE_DIRECTORY: UInt64 = 0x0000000000000010;
    pub const EFI_FILE_ARCHIVE: UInt64 = 0x0000000000000020;
    pub const EFI_FILE_VALID_ATTR: UInt64 = 0x0000000000000037;
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfiTime {
    year: UInt16,
    month: UInt8,
    day: UInt8,
    hour: UInt8,
    minute: UInt8,
    second: UInt8,
    pad1: UInt8,
    nanosecond: UInt32,
    time_zone: Int16,
    day_light: UInt8,
    pad2: UInt8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfiLoadedImageProtocol {
    revision: UInt32,
    parent_handle: EfiHandle,
    system_table: *const EfiSystemTable,

    pub device_handle: EfiHandle,
    file_path: usize,
    reserved: usize,

    load_options_size: UInt32,
    load_options: usize,

    image_base: *const u8,
    image_size: UInt64,
    image_code_type: EfiMemoryType,
    image_data_type: EfiMemoryType,
    unload: usize,
}

impl EfiLoadedImageProtocol {
    pub const GUID: EfiGuid = EfiGuid(
        0x5B1B31A1,
        0x9562,
        0x11d2,
        [0x8E, 0x3F, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B],
    );
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfiGraphicsOutputProtocol {
    pub query_mode: unsafe extern "efiapi" fn(
        this: *const EfiGraphicsOutputProtocol,
        mode_number: UInt32,
        size_of_info: *mut UIntN,
        info: *mut *const EfiGraphicsOutputModeInformation,
    ) -> EfiStatus,
    pub set_mode: unsafe extern "efiapi" fn(
        this: *const EfiGraphicsOutputProtocol,
        mode_number: UInt32,
    ) -> EfiStatus,
    pub blt: usize,
    pub mode: *const EfiGraphicsOutputProtocolMode,
}

impl EfiGraphicsOutputProtocol {
    pub const GUID: EfiGuid = EfiGuid(
        0x9042a9de,
        0x23dc,
        0x4a38,
        [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
    );
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfiGraphicsOutputProtocolMode {
    pub max_mode: UInt32,
    pub mode: UInt32,
    pub info: *const EfiGraphicsOutputModeInformation,
    pub size_of_info: UIntN,
    pub frame_buffer_base: EfiPhysicalAddress,
    pub frame_buffer_size: UIntN,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfiGraphicsOutputModeInformation {
    pub version: UInt32,
    pub horizontal_resolution: UInt32,
    pub vertical_resolution: UInt32,
    pub pixel_format: EfiGraphicsPixelFormat,
    pub pixel_information: EfiPixelBitmask,
    pub pixels_per_scanline: UInt32,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum EfiGraphicsPixelFormat {
    PixelRedGreenBlueReserved8BitPerColor = 0,
    PixelBlueGreenRedReserved8BitPerColor = 1,
    PixelBitMask = 2,
    PixelBltOnly = 3,
    PixelFormatMax = 4,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfiPixelBitmask {
    red_mask: UInt32,
    green_mask: UInt32,
    blue_mask: UInt32,
    reserved_mask: UInt32,
}
