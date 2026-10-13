use std::thread;

use terrakit::*;

mod common;
use common::last_error;

#[test]
fn panic_is_contained_and_diagnostic_is_available() {
    assert_eq!(tk_test_invoke_panic_guard(), TK_STATUS_INTERNAL_ERROR);
    let message = last_error();
    assert!(message.contains("panic caught"));
    assert!(message.contains("intentional ABI panic test"));
}

#[test]
fn error_messages_are_thread_local() {
    tk_clear_last_error();
    let child = thread::spawn(|| {
        assert_eq!(
            tk_get_abi_version(std::ptr::null_mut()),
            TK_STATUS_NULL_ARGUMENT
        );
        last_error()
    });
    let child_error = child.join().unwrap();
    assert!(child_error.contains("out_version"));

    let mut required = 99;
    assert_eq!(
        // SAFETY: querying accepts a null buffer with zero capacity, and
        // required points to writable test-owned storage.
        unsafe { tk_last_error_message_copy(std::ptr::null_mut(), 0, &mut required) },
        TK_STATUS_OK
    );
    assert_eq!(required, 1);
}
