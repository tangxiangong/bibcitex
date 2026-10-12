use crate::clipboard::{Clipboard, WriteReceipt};
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_app_kit::{NSPasteboard, NSPasteboardItem, NSPasteboardWriting};
use objc2_foundation::{NSArray, NSData, NSString};

pub(super) struct Pasteboard(pub Retained<NSPasteboard>);
type Items = Vec<Retained<NSPasteboardItem>>;

fn write_items(board: &NSPasteboard, items: &[Retained<NSPasteboardItem>]) -> bool {
    let objects: Vec<&ProtocolObject<dyn NSPasteboardWriting>> = items
        .iter()
        .map(|item| ProtocolObject::from_ref(&**item))
        .collect();
    board.clearContents();
    items.is_empty() || board.writeObjects(&NSArray::from_slice(&objects))
}

impl Clipboard for Pasteboard {
    type Snapshot = Items;

    fn snapshot(&mut self) -> Result<(Items, u64), String> {
        let board = &self.0;
        let revision = board.changeCount();
        let mut copies = Vec::new();
        if let Some(items) = board.pasteboardItems() {
            for item in items.iter() {
                let copy = NSPasteboardItem::new();
                for format in item.types().iter() {
                    let data = item
                        .dataForType(&format)
                        .ok_or("Cannot snapshot clipboard format")?;
                    // Materialize every representation, including images, files and rich text.
                    if !copy.setData_forType(&NSData::from_vec(data.to_vec()), &format) {
                        return Err("Cannot snapshot clipboard data".into());
                    }
                }
                copies.push(copy);
            }
        }
        if board.changeCount() != revision {
            return Err("Clipboard changed while taking snapshot".into());
        }
        Ok((copies, revision as u64))
    }

    fn write_text(&mut self, text: &str) -> Result<WriteReceipt, String> {
        let item = NSPasteboardItem::new();
        if !item.setString_forType(
            &NSString::from_str(text),
            &NSString::from_str("public.utf8-plain-text"),
        ) {
            return Err("Cannot prepare clipboard text".into());
        }
        let board = &self.0;
        let previous = board.changeCount() as u64;
        if !write_items(board, &[item]) {
            return Err("Cannot write clipboard text".into());
        }
        Ok(WriteReceipt {
            previous,
            owned: board.changeCount() as u64,
        })
    }

    fn restore(&mut self, original: Items, owned: u64) -> Result<bool, String> {
        let board = &self.0;
        if board.changeCount() as u64 != owned {
            return Ok(false);
        }
        if !write_items(board, &original) {
            return Err("Cannot restore clipboard data".into());
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2::rc::autoreleasepool;

    #[test]
    fn native_snapshot_restores_multiple_items_and_binary_formats() {
        autoreleasepool(|_| {
            let mut clipboard = Pasteboard(NSPasteboard::pasteboardWithUniqueName());
            let rich = NSPasteboardItem::new();
            assert!(rich.setString_forType(
                &NSString::from_str("original"),
                &NSString::from_str("public.utf8-plain-text")
            ));
            assert!(rich.setData_forType(
                &NSData::from_vec(b"<b>original</b>".to_vec()),
                &NSString::from_str("public.html")
            ));
            let binary = NSPasteboardItem::new();
            assert!(binary.setData_forType(
                &NSData::from_vec(vec![0, 255, 7]),
                &NSString::from_str("com.bibcitex.test-binary")
            ));
            assert!(write_items(&clipboard.0, &[rich, binary]));
            let (original, _) = clipboard.snapshot().unwrap();
            let owned = clipboard.write_text("citation").unwrap().owned;
            assert!(clipboard.restore(original, owned).unwrap());
            let (restored, _) = clipboard.snapshot().unwrap();
            assert_eq!(restored.len(), 2);
            assert_eq!(
                restored[0]
                    .dataForType(&NSString::from_str("public.html"))
                    .unwrap()
                    .to_vec(),
                b"<b>original</b>"
            );
            assert_eq!(
                restored[1]
                    .dataForType(&NSString::from_str("com.bibcitex.test-binary"))
                    .unwrap()
                    .to_vec(),
                vec![0, 255, 7]
            );
            clipboard.0.clearContents();
        });
    }

    #[test]
    fn native_change_count_protects_new_copy_of_identical_text() {
        autoreleasepool(|_| {
            let mut clipboard = Pasteboard(NSPasteboard::pasteboardWithUniqueName());
            clipboard.write_text("original").unwrap();
            let (original, _) = clipboard.snapshot().unwrap();
            let owned = clipboard.write_text("citation").unwrap().owned;
            let newer = clipboard.write_text("citation").unwrap().owned;
            assert_ne!(owned, newer);
            assert!(!clipboard.restore(original, owned).unwrap());
            assert_eq!(clipboard.0.changeCount() as u64, newer);
            clipboard.0.clearContents();
        });
    }

    #[test]
    fn native_empty_clipboard_is_restored_without_a_blank_text_item() {
        autoreleasepool(|_| {
            let mut clipboard = Pasteboard(NSPasteboard::pasteboardWithUniqueName());
            clipboard.0.clearContents();
            let (original, _) = clipboard.snapshot().unwrap();
            let owned = clipboard.write_text("citation").unwrap().owned;
            assert!(clipboard.restore(original, owned).unwrap());
            assert!(clipboard.snapshot().unwrap().0.is_empty());
        });
    }
}
