/*
 * Smoke test for the RVA C ABI. Loads a .rva, resolves several viewports and
 * reads a resource, then frees everything — exercising the full consumer path
 * that Swift/Kotlin/Flutter adapters use.
 */
#include "rva.h"
#include <stdio.h>
#include <stdlib.h>

static unsigned char *read_file(const char *path, size_t *len) {
    FILE *file = fopen(path, "rb");
    if (!file) {
        perror("fopen");
        exit(1);
    }
    fseek(file, 0, SEEK_END);
    long size = ftell(file);
    fseek(file, 0, SEEK_SET);
    unsigned char *buffer = malloc((size_t)size);
    if (fread(buffer, 1, (size_t)size, file) != (size_t)size) {
        perror("fread");
        exit(1);
    }
    fclose(file);
    *len = (size_t)size;
    return buffer;
}

int main(int argc, char **argv) {
    const char *path = argc > 1 ? argv[1] : "examples/node/hero.rva";

    size_t len = 0;
    unsigned char *data = read_file(path, &len);

    RvaHandle *handle = rva_open(data, len);
    if (!handle) {
        fprintf(stderr, "rva_open failed: %s\n", rva_last_error());
        return 1;
    }

    char *description = rva_describe(handle);
    printf("describe: %s\n", description);
    rva_free_string(description);

    uint32_t sizes[4][2] = {{1920, 500}, {1280, 720}, {1080, 1080}, {430, 932}};
    for (int i = 0; i < 4; i++) {
        char *scene = rva_resolve(handle, sizes[i][0], sizes[i][1]);
        if (!scene) {
            fprintf(stderr, "rva_resolve failed: %s\n", rva_last_error());
            return 1;
        }
        printf("%ux%u -> %.64s...\n", sizes[i][0], sizes[i][1], scene);
        rva_free_string(scene);
    }

    size_t resource_len = 0;
    uint8_t *resource = rva_resource(handle, "bgLandscape", &resource_len);
    if (!resource) {
        fprintf(stderr, "rva_resource failed: %s\n", rva_last_error());
        return 1;
    }
    char *relative = rva_relative_for(handle, "bgLandscape");
    printf(
        "resource bgLandscape: %zu bytes (present=%d), relative_for=%s\n",
        resource_len,
        rva_has_resource(handle, "bgLandscape"),
        relative ? relative : "(null)");
    rva_free_string(relative);
    rva_free_buffer(resource, resource_len);

    size_t png_len = 0;
    uint8_t *png = rva_render_png(handle, 1080, 1080, &png_len);
    if (!png) {
        fprintf(stderr, "rva_render_png failed: %s\n", rva_last_error());
        return 1;
    }
    printf(
        "rva_render_png: %zu bytes, png=%d, dims=%ux%u\n",
        png_len,
        png[0] == 0x89 && png[1] == 'P',
        (unsigned)((png[16] << 24) | (png[17] << 16) | (png[18] << 8) | png[19]),
        (unsigned)((png[20] << 24) | (png[21] << 16) | (png[22] << 8) | png[23]));
    rva_free_buffer(png, png_len);

    rva_free_handle(handle);
    free(data);
    return 0;
}
