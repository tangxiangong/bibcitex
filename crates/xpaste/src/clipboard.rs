//! Clipboard preservation policy, independent of UI and key injection.

#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) struct WriteReceipt {
    pub previous: u64,
    pub owned: u64,
}

#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) trait Clipboard {
    type Snapshot;
    fn snapshot(&mut self) -> Result<(Self::Snapshot, u64), String>;
    fn write_text(&mut self, text: &str) -> Result<WriteReceipt, String>;
    fn restore(&mut self, snapshot: Self::Snapshot, owned: u64) -> Result<bool, String>;
}

#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn paste(
    clipboard: &mut impl Clipboard,
    text: &str,
    inject: impl FnOnce() -> Result<(), String>,
    wait: impl FnOnce(std::time::Duration),
) -> Result<(), String> {
    // Preservation is best effort; it must never prevent ordinary paste.
    let original = clipboard.snapshot().ok();
    let written = clipboard.write_text(text)?;
    inject()?;
    // Key injection is not a receipt from the target app. This is a grace period only.
    wait(std::time::Duration::from_millis(200));
    // Do not report a delivered paste as failed and encourage a duplicate paste.
    if let Some((original, _)) = original.filter(|(_, revision)| *revision == written.previous) {
        let _ = clipboard.restore(original, written.owned);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    type Formats = Vec<(String, Vec<u8>)>;
    #[derive(Default)]
    struct State {
        formats: Formats,
        revision: u64,
        snapshot_fails: bool,
        write_fails: bool,
        restore_fails: bool,
        copy_before_write: bool,
        writes: usize,
        restores: usize,
    }
    struct Fake(Rc<RefCell<State>>);
    impl Clipboard for Fake {
        type Snapshot = Formats;
        fn snapshot(&mut self) -> Result<(Formats, u64), String> {
            let state = self.0.borrow();
            if state.snapshot_fails {
                return Err("unreadable format".into());
            }
            Ok((state.formats.clone(), state.revision))
        }
        fn write_text(&mut self, text: &str) -> Result<WriteReceipt, String> {
            let mut state = self.0.borrow_mut();
            if state.write_fails {
                return Err("write failed".into());
            }
            if state.copy_before_write {
                state.revision += 1;
            }
            let previous = state.revision;
            state.formats = vec![("text".into(), text.as_bytes().to_vec())];
            state.revision += 1;
            state.writes += 1;
            Ok(WriteReceipt {
                previous,
                owned: state.revision,
            })
        }
        fn restore(&mut self, original: Formats, owned: u64) -> Result<bool, String> {
            let mut state = self.0.borrow_mut();
            if state.revision != owned {
                return Ok(false);
            }
            if state.restore_fails {
                return Err("restore failed".into());
            }
            state.formats = original;
            state.revision += 1;
            state.restores += 1;
            Ok(true)
        }
    }
    fn setup(formats: Formats) -> (Fake, Rc<RefCell<State>>) {
        let state = Rc::new(RefCell::new(State {
            formats,
            ..State::default()
        }));
        (Fake(state.clone()), state)
    }

    #[test]
    fn restores_all_original_formats_after_grace_period() {
        let original = vec![
            ("html".into(), b"<b>original</b>".to_vec()),
            ("image".into(), vec![0, 255, 10]),
            ("file".into(), b"file:///test".to_vec()),
        ];
        let (mut clipboard, state) = setup(original.clone());
        paste(
            &mut clipboard,
            "key",
            || {
                assert_eq!(
                    state.borrow().formats,
                    vec![("text".into(), b"key".to_vec())]
                );
                Ok(())
            },
            |delay| {
                assert_eq!(delay, std::time::Duration::from_millis(200));
                assert_eq!(state.borrow().restores, 0);
            },
        )
        .unwrap();
        assert_eq!(state.borrow().formats, original);
    }

    #[test]
    fn restores_genuinely_empty_snapshot() {
        let (mut clipboard, state) = setup(vec![]);
        paste(&mut clipboard, "key", || Ok(()), |_| {}).unwrap();
        assert!(state.borrow().formats.is_empty());
    }

    #[test]
    fn snapshot_error_silently_falls_back_to_normal_paste() {
        let (mut clipboard, state) = setup(vec![]);
        state.borrow_mut().snapshot_fails = true;
        let result = paste(&mut clipboard, "key", || Ok(()), |_| {});
        assert_eq!(result, Ok(()));
        assert_eq!(
            state.borrow().formats,
            vec![("text".into(), b"key".to_vec())]
        );
        assert_eq!(state.borrow().restores, 0);
    }

    #[test]
    fn restoration_error_never_reports_paste_failure() {
        let (mut clipboard, state) = setup(vec![]);
        state.borrow_mut().restore_fails = true;
        assert_eq!(paste(&mut clipboard, "key", || Ok(()), |_| {}), Ok(()));
    }

    #[test]
    fn copy_between_snapshot_and_write_invalidates_the_old_snapshot() {
        let (mut clipboard, state) = setup(vec![("text".into(), b"old".to_vec())]);
        state.borrow_mut().copy_before_write = true;
        assert_eq!(paste(&mut clipboard, "key", || Ok(()), |_| {}), Ok(()));
        assert_eq!(state.borrow().restores, 0);
    }

    #[test]
    fn new_copy_of_identical_text_is_not_overwritten() {
        let (mut clipboard, state) = setup(vec![]);
        paste(
            &mut clipboard,
            "key",
            || Ok(()),
            |_| {
                // Same bytes, different OS clipboard generation.
                state.borrow_mut().revision += 1;
            },
        )
        .unwrap();
        assert_eq!(state.borrow().restores, 0);
        assert_eq!(
            state.borrow().formats,
            vec![("text".into(), b"key".to_vec())]
        );
    }

    #[test]
    fn newer_image_copy_is_not_overwritten() {
        let (mut clipboard, state) = setup(vec![]);
        let image = vec![("image".into(), vec![1, 2, 3])];
        paste(
            &mut clipboard,
            "key",
            || Ok(()),
            |_| {
                let mut state = state.borrow_mut();
                state.formats = image.clone();
                state.revision += 1;
            },
        )
        .unwrap();
        assert_eq!(state.borrow().formats, image);
    }

    #[test]
    fn injection_failure_keeps_manual_paste_available() {
        let (mut clipboard, state) = setup(vec![]);
        let result = paste(
            &mut clipboard,
            "key",
            || Err("focus failed".into()),
            |_| panic!("no restoration wait after failed injection"),
        );
        assert_eq!(result, Err("focus failed".into()));
        assert_eq!(
            state.borrow().formats,
            vec![("text".into(), b"key".to_vec())]
        );
    }

    #[test]
    fn failed_copy_does_not_inject_old_clipboard_content() {
        let (mut clipboard, state) = setup(vec![]);
        state.borrow_mut().write_fails = true;
        assert!(
            paste(
                &mut clipboard,
                "key",
                || panic!("must not inject"),
                |_| panic!("must not wait")
            )
            .is_err()
        );
    }
}
