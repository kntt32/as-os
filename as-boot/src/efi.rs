use core::ffi::c_void;
use core::mem;

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

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EfiGuid(pub UInt32, pub UInt16, pub UInt16, pub [UInt8; 8]);

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EfiTableHeader {
    signature: UInt64,
    revision: UInt32,
    header_size: UInt32,
    crc32: UInt32,
    reserved: UInt32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EfiSystemTable {
    hdr: EfiTableHeader,
    firmware_vendor: *const Char16,
    firmware_revision: UInt32,
    console_in_handle: EfiHandle,
    con_in: *mut EfiSimpleTextInputProtocol,
    console_out_handle: EfiHandle,
    con_out: *mut EfiSimpleTextOutputProtocol,
    standard_error_handle: EfiHandle,
    std_err: *mut EfiSimpleTextOutputProtocol,
    runtime_services: *mut EfiRuntimeServices,
    boot_services: *mut EfiBootServices,
    number_of_table_entries: UIntN,
    configuration_table: *const EfiConfigurationTable,
}

impl EfiSystemTable {
    pub fn con_out(&mut self) -> &mut EfiSimpleTextOutputProtocol {
        unsafe {
            self.con_out.as_mut().expect("expected non-null pointer")
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EfiConfigurationTable {
    vendor_guid: EfiGuid,
    vendor_table: *const c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EfiInputKey {
    scan_code: UInt16,
    unicode_char: Char16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EfiSimpleTextInputProtocol {
    reset: extern "efiapi" fn(this: *mut EfiSimpleTextInputProtocol, extended_verification: Boolean) -> EfiStatus,
    read_key_stroke: extern "efiapi" fn(this: *mut EfiSimpleTextInputProtocol, key: *mut EfiInputKey) -> EfiStatus,
    wait_for_key: EfiEvent,
}

impl EfiSimpleTextInputProtocol {
    pub const GUID: EfiGuid = EfiGuid(0x387477c1, 0x69c7, 0x11d2, [0x8e,0x39,0x00,0xa0,0xc9,0x69,0x72,0x3b]);
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EfiSimpleTextOutputProtocol {
    reset: extern "efiapi" fn(this: *mut EfiSimpleTextOutputProtocol, extended_verification: Boolean) -> EfiStatus,
    output_string: extern "efiapi" fn(this: *mut EfiSimpleTextOutputProtocol, string: *const Char16) -> EfiStatus,
    test_string: extern "efiapi" fn(this: *mut EfiSimpleTextOutputProtocol, string: *const Char16) -> EfiStatus,
    query_mode: extern "efiapi" fn(this: *mut EfiSimpleTextOutputProtocol, mode_number: UIntN, columns: *mut UIntN, rows: *mut UIntN) -> EfiStatus,
    set_mode: extern "efiapi" fn(this: *mut EfiSimpleTextOutputProtocol, mode_number: UIntN) -> EfiStatus,
    set_attribute: extern "efiapi" fn(this: *mut EfiSimpleTextOutputProtocol, attribute: UIntN) -> EfiStatus,
    clear_screen: extern "efiapi" fn(this: *mut EfiSimpleTextOutputProtocol) -> EfiStatus,
    set_cursor_position: extern "efiapi" fn(this: *mut EfiSimpleTextOutputProtocol, column: UIntN, row: UIntN) -> EfiStatus,
    enable_cursor: extern "efiapi" fn(this: *mut EfiSimpleTextOutputProtocol, visible: Boolean) -> EfiStatus,
    mode: *const SimpleTextOutputMode,
}

impl EfiSimpleTextOutputProtocol {
    pub const GUID: EfiGuid = EfiGuid(0x387477c2, 0x69c7, 0x11d2, [0x8e,0x39,0x00,0xa0,0xc9,0x69,0x72,0x3b]);

    pub fn output_string(&mut self, string: &str) -> EfiStatus {
        let mut string_encode_utf16 = string.encode_utf16();

        loop {
            let mut string_utf16_buff: [u16; 1024] = unsafe { mem::zeroed() };
            let mut string_utf16_len: usize = 0;

            while string_utf16_len < string_utf16_buff.len() - 1 {
                let Some(utf16_code) = string_encode_utf16.next() else {
                    break;
                };

                string_utf16_buff[string_utf16_len] = utf16_code;
                string_utf16_len += 1;
            }
            string_utf16_buff[string_utf16_buff.len() -1] = 0;

            if string_utf16_len == 0 {
                break EFI_STATUS_SUCCESS;
            }

            let status = (self.output_string)(self as *mut Self, &string_utf16_buff as *const u16);
            if status != EFI_STATUS_SUCCESS {
                break status;
            }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimpleTextOutputMode {
    max_mode: Int32,
    mode: Int32,
    attribute: Int32,
    cursor_column: Int32,
    cursor_row: Int32,
    cursor_visible: Boolean,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EfiRuntimeServices {
    hdr: EfiTableHeader,

    get_time: *const c_void,
    set_time: *const c_void,
    get_wakeup_time: *const c_void,
    set_wakeup_time: *const c_void,

    set_virtual_address_map: extern "efiapi" fn(memory_map_size: UIntN, descriptor_size: UIntN, descriptor_version: UIntN, virtual_map: *const EfiMemoryDescriptor) -> EfiStatus,
    convert_pointer: extern "efiapi" fn(debug_disposition: UIntN, address: *mut *mut c_void) -> EfiStatus,

    get_variable: extern "efiapi" fn(variable_name: *const Char16, vendor_guid: *const EfiGuid, attributes: *mut UInt32, data_size: *mut UIntN, data: *mut c_void) -> EfiStatus,
    get_next_variable_name: extern "efiapi" fn(variable_name_size: *mut UIntN, variable_name: *mut Char16, vendor_guid: *mut EfiGuid) -> EfiStatus,
    set_variable: extern "efiapi" fn(variable_name: *const Char16, vendor_guid: *const EfiGuid, attributes: *const UInt32, data_size: UIntN, data: *const c_void) -> EfiStatus,

    get_next_high_monotonic_count: *const c_void,
    reset_system: extern "efiapi" fn(reset_type: EfiResetType, reset_status: EfiStatus, data_size: UIntN, reset_data: *const c_void) -> !,
    update_capsule: *const c_void,
    query_capsule_capabilities: *const c_void,
    query_variable_info: *const c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EfiResetType {
    EfiResetCold = 0,
    EfiResetWarm = 1,
    EfiResetShutdown = 2,
    EfiResetPlatformSpecific = 3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EfiBootServices {
    hdr: EfiTableHeader,
    
    raise_tpl: *const c_void,
    restore_tpl: *const c_void,

    allocate_pages: extern "efiapi" fn(r#type: EfiAllocateType, memory_type: EfiMemoryType, pages: UIntN, memory: *mut EfiPhysicalAddress) -> EfiStatus,
    free_pages: extern "efiapi" fn(memory: EfiPhysicalAddress, pages: UIntN) -> EfiStatus,
    get_memory_map: extern "efiapi" fn(memory_map_size: *mut UIntN, memory_map: *mut EfiMemoryDescriptor, map_key: *mut UIntN, descriptor_size: *mut UIntN, descriptor_version: *mut UInt32) -> EfiStatus,
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
    handle_protocol: extern "efiapi" fn(handle: EfiHandle, protocol: *const EfiGuid, interface: *mut *mut c_void) -> EfiStatus,
    _reserved: *const c_void,
    register_protocol_notify: *const c_void,
    locate_handle: extern "efiapi" fn(search_type: EfiLocateSearchType, protocol: *const EfiGuid, search_key: *const c_void, buffer_size: *mut UIntN, buffer: *mut EfiHandle) -> EfiStatus,
    locate_device_path: *const c_void,
    install_configurtaion_table: *const c_void,

    load_image: *const c_void,
    start_image: *const c_void,
    exit: *const c_void,
    unload_image: *const c_void,
    exit_boot_services: extern "efiapi" fn(image_handle: EfiHandle, map_key: UIntN) -> EfiStatus,

    get_next_monotonic_count: *const c_void,
    stall: *const c_void,
    set_watchdog_timer: *const c_void,

    connect_controller: *const c_void,
    disconnect_controller: *const c_void, 
    
    open_protocol: extern "efiapi" fn(handle: EfiHandle, protocol: *const EfiGuid, interface: *mut*mut c_void, agent_handle: EfiHandle, controller_handle: EfiHandle, attributes: UInt32) -> EfiStatus,
    close_protocol: extern "efiapi" fn(handle: EfiHandle, protocol: *const EfiGuid, agent_handle: EfiHandle, controller_handle: *const EfiHandle) -> EfiStatus,
    open_protocol_information: *const c_void,

    protocols_per_handle: *const c_void,
    locate_handle_buffer: *const c_void,
    locate_protocol: extern "efiapi" fn(protocol: *const EfiGuid, registration: *const c_void, interface: *mut *mut c_void) -> EfiStatus,
    install_multiple_protocol_interfaces: *const c_void,
    uninstall_multiple_protocol_interfaces: *const c_void,
    
    calculate_crc32: *const c_void,

    copy_mem: *const c_void,
    set_mem: *const c_void,
    create_event_ex: *const c_void,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EfiLocateSearchType {
    AllHandles = 0,
    ByRegisterNotify = 1,
    ByProtocol = 2,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EfiAllocateType {
    AllocateAnyPages = 0,
    AllocateMaxAddress = 1,
    AllocateAddress = 2,
    MaxAllocateType = 3,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

