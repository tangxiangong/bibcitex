use crate::clipboard::{Clipboard, WriteReceipt};
use windows::Win32::{
    Foundation::{GetLastError, GlobalFree, HANDLE, HGLOBAL, HWND, SetLastError, WIN32_ERROR},
    System::{
        DataExchange::{
            CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData,
            GetClipboardSequenceNumber, OpenClipboard, SetClipboardData,
        },
        Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock},
    },
};

// A message-only window gives this operation an explicit clipboard owner.
// It is never shown and is created/destroyed on the paste worker thread.
pub(super) struct Pasteboard(HWND);
impl Pasteboard {
    fn revision(&self) -> u64 {
        unsafe { GetClipboardSequenceNumber().into() }
    }

    pub fn new() -> Result<Self, String> {
        use windows::{Win32::UI::WindowsAndMessaging::*, core::w};
        unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                w!("BibCiTeX clipboard"),
                WINDOW_STYLE::default(),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                None,
                None,
            )
            .map(Self)
            .map_err(|e| e.to_string())
        }
    }
}
impl Drop for Pasteboard {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(self.0);
        }
    }
}
struct OpenBoard;
impl OpenBoard {
    fn new(owner: HWND) -> Result<Self, String> {
        let mut attempts = 10;
        loop {
            match unsafe { OpenClipboard(Some(owner)) } {
                Ok(()) => return Ok(Self),
                Err(error) => {
                    attempts -= 1;
                    if attempts == 0 {
                        return Err(error.to_string());
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            }
        }
    }
}
impl Drop for OpenBoard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

struct Allocation(HGLOBAL);
impl Allocation {
    fn new(bytes: &[u8]) -> Result<Self, String> {
        unsafe {
            let allocation =
                Self(GlobalAlloc(GMEM_MOVEABLE, bytes.len()).map_err(|e| e.to_string())?);
            let pointer = GlobalLock(allocation.0);
            if pointer.is_null() {
                return Err("Cannot lock clipboard allocation".into());
            }
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), pointer.cast(), bytes.len());
            let _ = GlobalUnlock(allocation.0);
            Ok(allocation)
        }
    }
}
impl Drop for Allocation {
    fn drop(&mut self) {
        unsafe {
            let _ = GlobalFree(Some(self.0));
        }
    }
}

impl Clipboard for Pasteboard {
    type Snapshot = Vec<(u32, Vec<u8>)>;

    fn snapshot(&mut self) -> Result<(Self::Snapshot, u64), String> {
        let _open = OpenBoard::new(self.0)?;
        let mut snapshot = Vec::new();
        let mut format = 0;
        unsafe {
            loop {
                SetLastError(WIN32_ERROR(0));
                format = EnumClipboardFormats(format);
                if format == 0 {
                    if GetLastError().0 != 0 {
                        return Err("Cannot enumerate clipboard formats".into());
                    }
                    break;
                }
                // These handles are not HGLOBAL data. Decline preservation rather than
                // flattening or partially restoring a clipboard containing them.
                if matches!(format, 2 | 3 | 9 | 14 | 0x80..=0x8e | 0x300..=0x3ff) {
                    return Err("Clipboard contains non-memory formats".into());
                }
                let handle = HGLOBAL(GetClipboardData(format).map_err(|e| e.to_string())?.0);
                let size = GlobalSize(handle);
                if size == 0 {
                    return Err("Cannot read clipboard format size".into());
                }
                let pointer = GlobalLock(handle);
                if pointer.is_null() {
                    return Err("Cannot snapshot clipboard format".into());
                }
                let bytes = std::slice::from_raw_parts(pointer.cast::<u8>(), size).to_vec();
                let _ = GlobalUnlock(handle);
                snapshot.push((format, bytes));
            }
        }
        Ok((snapshot, self.revision()))
    }

    fn write_text(&mut self, text: &str) -> Result<WriteReceipt, String> {
        let bytes: Vec<u8> = text
            .encode_utf16()
            .chain(std::iter::once(0))
            .flat_map(u16::to_le_bytes)
            .collect();
        let allocation = Allocation::new(&bytes)?;
        let _open = OpenBoard::new(self.0)?;
        let previous = self.revision();
        unsafe {
            EmptyClipboard().map_err(|e| e.to_string())?;
            SetClipboardData(13, Some(HANDLE(allocation.0.0))).map_err(|e| e.to_string())?;
            std::mem::forget(allocation);
        }
        // No competing writer can change the board between the write and this read.
        Ok(WriteReceipt {
            previous,
            owned: self.revision(),
        })
    }

    fn restore(&mut self, snapshot: Self::Snapshot, owned: u64) -> Result<bool, String> {
        // Allocate everything before changing the board. Keep the OS clipboard lock
        // across both the sequence check and the restoration.
        let allocations = snapshot
            .iter()
            .map(|(format, bytes)| Allocation::new(bytes).map(|allocation| (*format, allocation)))
            .collect::<Result<Vec<_>, _>>()?;
        let _open = OpenBoard::new(self.0)?;
        if self.revision() != owned {
            return Ok(false);
        }
        unsafe {
            EmptyClipboard().map_err(|e| e.to_string())?;
            for (format, allocation) in allocations {
                SetClipboardData(format, Some(HANDLE(allocation.0.0)))
                    .map_err(|e| e.to_string())?;
                // Windows now owns this allocation.
                std::mem::forget(allocation);
            }
        }
        Ok(true)
    }
}
