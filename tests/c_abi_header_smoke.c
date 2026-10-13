#include "terrakit.h"

/* Compile-only check: these public names must be visible to a C compiler. */
static tk_status_t (*const abi_version_fn)(tk_version_t *) = tk_get_abi_version;
static tk_status_t (*const scatter_view_fn)(
    const tk_generation_result_t *, uint64_t, tk_scatter_points_view_t *) =
    tk_generation_result_get_scatter_points;

int main(void) {
    tk_version_t version = {0, 0, 0};
    (void)scatter_view_fn;
    return abi_version_fn(&version);
}
