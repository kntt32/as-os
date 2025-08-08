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
    std_err: *mut EfiSimpleTextOutputProtocol,/*
    runtime_services: *mut EfiRuntimeService,
    boot_services: *mut BootServices,
    number_of_table_entries: UIntN,
    configuration_table: *EfiConfigurationTable,*/
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
pub struct EfiInputKey {
    scan_code: UInt16,
    unicode_char: Char16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EfiSimpleTextInputProtocol {
    reset: extern "efiapi" fn(this: *mut EfiSimpleTextInputProtocol, extended_vertification: Boolean) -> EfiStatus,
    read_key_stroke: extern "efiapi" fn(this: *mut EfiSimpleTextInputProtocol, key: *mut EfiInputKey) -> EfiStatus,
    wait_for_key: EfiEvent,
}

impl EfiSimpleTextInputProtocol {
    pub const GUID: EfiGuid = EfiGuid(0x387477c1, 0x69c7, 0x11d2, [0x8e,0x39,0x00,0xa0,0xc9,0x69,0x72,0x3b]);
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EfiSimpleTextOutputProtocol {
    reset: extern "efiapi" fn(this: *mut EfiSimpleTextOutputProtocol, extended_vertification: Boolean) -> EfiStatus,
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
        let mut string_utf16_buff: [u16; 1024] = unsafe { mem::zeroed() };
        let mut string_utf16_len: usize = 0;

        for c in string.encode_utf16() {
            string_utf16_buff[string_utf16_len] = c;
            string_utf16_len += 1;
        }

        let self_ptr = self as *mut Self;

        (self.output_string)(self_ptr, &string_utf16_buff as *const u16)
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
/*
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EfiBootServices {
    hdr: EfiTableHeader,
    raise_tpl: 
    restore_tpl: 
    allocate_pages: 
    free_pages:
    get_memory_map:
    e
}:*/
