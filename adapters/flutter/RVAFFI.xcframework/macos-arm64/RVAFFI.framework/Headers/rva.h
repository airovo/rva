/*
 * RVA core C ABI.
 *
 * The single native bridge for Swift, Kotlin, Flutter and C adapters. The core
 * decides WHAT to draw; adapters draw the returned ResolvedScene JSON with their
 * platform graphics stack.
 *
 * Memory rules:
 *   - char * results (except rva_last_error) are owned by the caller and must be
 *     released with rva_free_string.
 *   - uint8_t * results must be released with rva_free_buffer(ptr, len).
 *   - rva_last_error returns a borrowed, thread-local string; do not free.
 *   - On failure, functions return NULL/0/false and set rva_last_error.
 */
#ifndef RVA_H
#define RVA_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct RvaHandle RvaHandle;

/* Parse and integrity-validate a .rva package from memory. */
RvaHandle *rva_open(const uint8_t *data, size_t len);

/* Release a handle created by rva_open. */
void rva_free_handle(RvaHandle *handle);

/* Human-readable asset summary. Caller frees with rva_free_string. */
char *rva_describe(const RvaHandle *handle);

/* ResolvedScene JSON for a logical viewport. Caller frees with rva_free_string. */
char *rva_resolve(const RvaHandle *handle, uint32_t width, uint32_t height);

/* Raw resource bytes; sets *out_len. Caller frees with rva_free_buffer. */
uint8_t *rva_resource(const RvaHandle *handle, const char *name, size_t *out_len);

/* Resolve + rasterize to PNG bytes; sets *out_len. Caller frees with rva_free_buffer. */
uint8_t *rva_render_png(const RvaHandle *handle, uint32_t width, uint32_t height, size_t *out_len);

/* Whether a resource reference exists in the package. */
bool rva_has_resource(const RvaHandle *handle, const char *name);

/* Path a reference resolves to (for MIME detection). Caller frees. */
char *rva_relative_for(const RvaHandle *handle, const char *name);

/* Borrowed, thread-local last error message. Do not free. */
const char *rva_last_error(void);

void rva_free_string(char *ptr);
void rva_free_buffer(uint8_t *ptr, size_t len);

#ifdef __cplusplus
}
#endif

#endif /* RVA_H */
