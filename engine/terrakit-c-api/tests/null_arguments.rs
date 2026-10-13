mod common;

use std::ptr;

use common::*;
use terrakit::*;

#[test]
fn null_required_arguments_return_null_argument() {
    let mut count = 0_usize;
    assert_eq!(tk_get_abi_version(ptr::null_mut()), TK_STATUS_NULL_ARGUMENT);
    assert!(last_error().contains("out_version"));

    let mut registry = create_registry();
    assert_eq!(
        tk_stage_registry_get_schema_count(ptr::null(), &mut count),
        TK_STATUS_NULL_ARGUMENT
    );
    assert_eq!(
        tk_stage_registry_get_schema_count(registry, ptr::null_mut()),
        TK_STATUS_NULL_ARGUMENT
    );
    assert_eq!(
        tk_pipeline_assembler_create(ptr::null(), ptr::null_mut()),
        TK_STATUS_NULL_ARGUMENT
    );
    assert_eq!(
        tk_stage_registry_destroy(ptr::null_mut()),
        TK_STATUS_NULL_ARGUMENT
    );
    destroy_registry(&mut registry);
}

#[test]
fn last_error_copy_obeys_query_and_buffer_contract() {
    assert_eq!(tk_get_abi_version(ptr::null_mut()), TK_STATUS_NULL_ARGUMENT);
    let mut required = 0;
    assert_eq!(
        // SAFETY: querying accepts a null buffer with zero capacity, and
        // required points to writable test-owned storage.
        unsafe { tk_last_error_message_copy(ptr::null_mut(), 0, &mut required) },
        TK_STATUS_OK
    );
    assert!(required > 1);

    let mut tiny = [0_i8; 2];
    assert_eq!(
        // SAFETY: tiny is writable for tiny.len() bytes, and required points
        // to writable test-owned storage.
        unsafe { tk_last_error_message_copy(tiny.as_mut_ptr(), tiny.len(), &mut required) },
        TK_STATUS_BUFFER_TOO_SMALL
    );
    assert!(last_error().contains("out_version"));

    assert_eq!(
        // SAFETY: this intentionally exercises the ABI's null-buffer status;
        // required still points to writable test-owned storage.
        unsafe { tk_last_error_message_copy(ptr::null_mut(), 4, &mut required) },
        TK_STATUS_NULL_ARGUMENT
    );
    assert!(last_error().contains("out_version"));

    tk_clear_last_error();
    assert_eq!(
        // SAFETY: querying accepts a null buffer with zero capacity, and
        // required points to writable test-owned storage.
        unsafe { tk_last_error_message_copy(ptr::null_mut(), 0, &mut required) },
        TK_STATUS_OK
    );
    assert_eq!(required, 1);
}
