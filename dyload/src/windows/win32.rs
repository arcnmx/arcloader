pub mod Foundation {
	use crate::windows::{WinError, WinResult, core::HRESULT};

	pub type BYTE = u8;
	pub type WORD = u16;
	pub type DWORD = u32;
	pub type LONG = i32;
	pub type ULONGLONG = u64;

	#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
	#[repr(transparent)]
	pub struct WIN32_ERROR(pub u32);

	impl WIN32_ERROR {
		pub fn to_hresult(self) -> HRESULT {
			HRESULT(-(self.0 as i32))
		}

		pub fn is_ok(self) -> bool {
			self.0 == 0
		}

		pub fn is_err(self) -> bool {
			!self.is_ok()
		}

		pub fn ok(self) -> WinResult<()> {
			match self.is_ok() {
				true => Ok(()),
				false => Err(WinError::from_hresult(self.to_hresult())),
			}
		}
	}

	pub const ERROR_SUCCESS: WIN32_ERROR = WIN32_ERROR(0);
	pub const ERROR_INVALID_FUNCTION: WIN32_ERROR = WIN32_ERROR(1);
	pub const ERROR_FILE_NOT_FOUND: WIN32_ERROR = WIN32_ERROR(2);
	pub const ERROR_PATH_NOT_FOUND: WIN32_ERROR = WIN32_ERROR(3);
	pub const ERROR_TOO_MANY_OPEN_FILES: WIN32_ERROR = WIN32_ERROR(4);
	pub const ERROR_ACCESS_DENIED: WIN32_ERROR = WIN32_ERROR(5);
	pub const ERROR_INVALID_HANDLE: WIN32_ERROR = WIN32_ERROR(6);
	pub const ERROR_BAD_FORMAT: WIN32_ERROR = WIN32_ERROR(11);
	pub const ERROR_INVALID_ACCESS: WIN32_ERROR = WIN32_ERROR(12);
	pub const ERROR_INVALID_DATA: WIN32_ERROR = WIN32_ERROR(13);
	pub const ERROR_NOT_READY: WIN32_ERROR = WIN32_ERROR(21);
	pub const ERROR_BAD_COMMAND: WIN32_ERROR = WIN32_ERROR(22);
	pub const ERROR_BAD_CRC: WIN32_ERROR = WIN32_ERROR(23);
	pub const ERROR_BAD_LENGTH: WIN32_ERROR = WIN32_ERROR(24);
	pub const ERROR_HANDLE_EOF: WIN32_ERROR = WIN32_ERROR(38);
	pub const ERROR_NOT_SUPPORTED: WIN32_ERROR = WIN32_ERROR(50);
	pub const ERROR_NETWORK_BUSY: WIN32_ERROR = WIN32_ERROR(54);
	pub const ERROR_FILE_EXISTS: WIN32_ERROR = WIN32_ERROR(80);
	pub const ERROR_BROKEN_PIPE: WIN32_ERROR = WIN32_ERROR(109);
	pub const ERROR_BUFFER_OVERFLOW: WIN32_ERROR = WIN32_ERROR(111);
	pub const ERROR_CALL_NOT_IMPLEMENTED: WIN32_ERROR = WIN32_ERROR(120);
	pub const ERROR_INSUFFICIENT_BUFFER: WIN32_ERROR = WIN32_ERROR(122);
	pub const ERROR_INVALID_NAME: WIN32_ERROR = WIN32_ERROR(123);
	pub const ERROR_MOD_NOT_FOUND: WIN32_ERROR = WIN32_ERROR(126);
	pub const ERROR_PROC_NOT_FOUND: WIN32_ERROR = WIN32_ERROR(127);
	pub const ERROR_BAD_ARGUMENTS: WIN32_ERROR = WIN32_ERROR(160);
	pub const ERROR_BAD_PATHNAME: WIN32_ERROR = WIN32_ERROR(161);
	pub const ERROR_BUSY: WIN32_ERROR = WIN32_ERROR(170);
}

pub mod System {
	pub mod Diagnostics {
		pub mod Debug {
			use super::super::super::Foundation::*;

			#[derive(Debug, Copy, Clone, Default)]
			#[repr(C)]
			pub struct IMAGE_NT_HEADERS64 {
				pub Signature: DWORD,
				pub FileHeader: IMAGE_FILE_HEADER,
				pub OptionalHeader: IMAGE_OPTIONAL_HEADER64,
			}

			#[derive(Debug, Copy, Clone, Default)]
			#[repr(C)]
			pub struct IMAGE_FILE_HEADER {
				pub Machine: WORD,
				pub NumberOfSections: WORD,
				pub TimeDateStamp: DWORD,
				pub PointerToSymbolTable: DWORD,
				pub NumberOfSymbols: DWORD,
				pub SizeOfOptionalHeader: WORD,
				pub Characteristics: WORD,
			}

			#[derive(Debug, Copy, Clone, Default)]
			#[repr(C)]
			pub struct IMAGE_OPTIONAL_HEADER64 {
				pub Magic: WORD,
				pub MajorLinkerVersion: BYTE,
				pub MinorLinkerVersion: BYTE,
				pub SizeOfCode: DWORD,
				pub SizeOfInitializedData: DWORD,
				pub SizeOfUninitializedData: DWORD,
				pub AddressOfEntryPoint: DWORD,
				pub BaseOfCode: DWORD,
				pub ImageBase: ULONGLONG,
				pub SectionAlignment: DWORD,
				pub FileAlignment: DWORD,
				pub MajorOperatingSystemVersion: WORD,
				pub MinorOperatingSystemVersion: WORD,
				pub MajorImageVersion: WORD,
				pub MinorImageVersion: WORD,
				pub MajorSubsystemVersion: WORD,
				pub MinorSubsystemVersion: WORD,
				pub Win32VersionValue: DWORD,
				pub SizeOfImage: DWORD,
				pub SizeOfHeaders: DWORD,
				pub CheckSum: DWORD,
				pub Subsystem: WORD,
				pub DllCharacteristics: WORD,
				pub SizeOfStackReserve: ULONGLONG,
				pub SizeOfStackCommit: ULONGLONG,
				pub SizeOfHeapReserve: ULONGLONG,
				pub SizeOfHeapCommit: ULONGLONG,
				pub LoaderFlags: DWORD,
				pub NumberOfRvaAndSizes: DWORD,
				pub DataDirectory: [IMAGE_DATA_DIRECTORY; IMAGE_NUMBEROF_DIRECTORY_ENTRIES],
			}
			#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
			#[repr(transparent)]
			pub struct IMAGE_DIRECTORY_ENTRY(pub u16);
			pub const IMAGE_DIRECTORY_ENTRY_EXPORT: IMAGE_DIRECTORY_ENTRY = IMAGE_DIRECTORY_ENTRY(0);
			pub const IMAGE_DIRECTORY_ENTRY_IMPORT: IMAGE_DIRECTORY_ENTRY = IMAGE_DIRECTORY_ENTRY(1);
			pub const IMAGE_DIRECTORY_ENTRY_RESOURCE: IMAGE_DIRECTORY_ENTRY = IMAGE_DIRECTORY_ENTRY(2);
			pub const IMAGE_DIRECTORY_ENTRY_EXCEPTION: IMAGE_DIRECTORY_ENTRY = IMAGE_DIRECTORY_ENTRY(3);
			pub const IMAGE_DIRECTORY_ENTRY_SECURITY: IMAGE_DIRECTORY_ENTRY = IMAGE_DIRECTORY_ENTRY(4);
			pub const IMAGE_DIRECTORY_ENTRY_BASERELOC: IMAGE_DIRECTORY_ENTRY = IMAGE_DIRECTORY_ENTRY(5);
			pub const IMAGE_DIRECTORY_ENTRY_DEBUG: IMAGE_DIRECTORY_ENTRY = IMAGE_DIRECTORY_ENTRY(6);
			pub const IMAGE_DIRECTORY_ENTRY_ARCHITECTURE: IMAGE_DIRECTORY_ENTRY = IMAGE_DIRECTORY_ENTRY(7);
			pub const IMAGE_DIRECTORY_ENTRY_GLOBALPTR: IMAGE_DIRECTORY_ENTRY = IMAGE_DIRECTORY_ENTRY(8);
			pub const IMAGE_DIRECTORY_ENTRY_TLS: IMAGE_DIRECTORY_ENTRY = IMAGE_DIRECTORY_ENTRY(9);
			pub const IMAGE_DIRECTORY_ENTRY_LOAD_CONFIG: IMAGE_DIRECTORY_ENTRY = IMAGE_DIRECTORY_ENTRY(10);
			pub const IMAGE_DIRECTORY_ENTRY_BOUND_IMPORT: IMAGE_DIRECTORY_ENTRY = IMAGE_DIRECTORY_ENTRY(11);
			pub const IMAGE_DIRECTORY_ENTRY_IAT: IMAGE_DIRECTORY_ENTRY = IMAGE_DIRECTORY_ENTRY(12);
			pub const IMAGE_DIRECTORY_ENTRY_DELAY_IMPORT: IMAGE_DIRECTORY_ENTRY = IMAGE_DIRECTORY_ENTRY(13);
			pub const IMAGE_DIRECTORY_ENTRY_COM_DESCRIPTOR: IMAGE_DIRECTORY_ENTRY = IMAGE_DIRECTORY_ENTRY(14);
			const IMAGE_NUMBEROF_DIRECTORY_ENTRIES: usize = 16;

			#[derive(Debug, Copy, Clone, Default)]
			#[repr(C)]
			pub struct IMAGE_DATA_DIRECTORY {
				pub VirtualAddress: DWORD,
				pub Size: DWORD,
			}

			#[derive(Debug, Copy, Clone, Default)]
			#[repr(C)]
			pub struct IMAGE_SECTION_HEADER {
				pub Name: [BYTE; IMAGE_SIZEOF_SHORT_NAME as usize],
				/// See also: [self.VirtualSize()]
				pub Misc: DWORD,
				pub VirtualAddress: DWORD,
				pub SizeOfRawData: DWORD,
				pub PointerToRawData: DWORD,
				pub PointerToRelocations: DWORD,
				pub PointerToLinenumbers: DWORD,
				pub NumberOfRelocations: WORD,
				pub NumberOfLinenumbers: WORD,
				pub Characteristics: DWORD,
			}
			pub const IMAGE_SIZEOF_SHORT_NAME: u32 = 8;

			impl IMAGE_SECTION_HEADER {
				pub fn PhysicalAddress(&self) -> DWORD {
					self.Misc
				}
				pub fn VirtualSize(&self) -> DWORD {
					self.Misc
				}
			}
		}
	}

	pub mod SystemInformation {
		use super::super::Foundation::*;

		pub const IMAGE_FILE_MACHINE_UNKNOWN: WORD = 0;
		pub const IMAGE_FILE_MACHINE_TARGET_HOST: WORD = 1;
		pub const IMAGE_FILE_MACHINE_I386: WORD = 0x014c;
		pub const IMAGE_FILE_MACHINE_AMD64: WORD = 0x8664;
		pub const IMAGE_FILE_MACHINE_ARM64: WORD = 0xAA64;
	}

	pub mod SystemServices {
		use super::super::Foundation::*;

		pub const IMAGE_DOS_SIGNATURE: u16 = u16::from_le_bytes(*b"MZ");
		pub const IMAGE_NT_SIGNATURE: u32 = u32::from_le_bytes(*b"PE\0\0");

		#[derive(Debug, Copy, Clone, Default)]
		#[repr(C)]
		pub struct IMAGE_DOS_HEADER {
			pub e_magic: WORD,
			pub e_cblp: WORD,
			pub e_cp: WORD,
			pub e_crlc: WORD,
			pub e_cparhdr: WORD,
			pub e_minalloc: WORD,
			pub e_maxalloc: WORD,
			pub e_ss: WORD,
			pub e_sp: WORD,
			pub e_csum: WORD,
			pub e_ip: WORD,
			pub e_cs: WORD,
			pub e_lfarlc: WORD,
			pub e_ovno: WORD,
			pub e_res: [WORD; 4],
			pub e_oemid: WORD,
			pub e_oeminfo: WORD,
			// 36
			pub e_res2: [WORD; 10],
			// 46
			pub e_lfanew: LONG,
		}

		#[derive(Debug, Copy, Clone, Default)]
		#[repr(C)]
		pub struct IMAGE_EXPORT_DIRECTORY {
			pub Characteristics: DWORD,
			pub TimeDateStamp: DWORD,
			pub MajorVersion: WORD,
			pub MinorVersion: WORD,
			pub Name: DWORD,
			pub Base: DWORD,
			pub NumberOfFunctions: DWORD,
			pub NumberOfNames: DWORD,
			pub AddressOfFunctions: DWORD,
			pub AddressOfNames: DWORD,
			pub AddressOfNameOrdinals: DWORD,
		}
	}
}
