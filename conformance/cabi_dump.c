/*
 * C-ABI conformance runner: resolves a fixture at every size listed in a sizes
 * file and prints "<w> <h> <resolved-scene-json>" per line. The JSON is exactly
 * what rva_resolve() returns, so it can be byte-compared with the native CLI
 * (--compact) and the WASM runtime.
 *
 * usage: cabi_dump <asset.rva> <sizes.txt>
 */
#include "rva.h"
#include <stdio.h>
#include <stdlib.h>

static unsigned char *read_all(const char *path, size_t *len) {
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
    if (argc < 3) {
        fprintf(stderr, "usage: cabi_dump <asset.rva> <sizes.txt>\n");
        return 2;
    }

    size_t len = 0;
    unsigned char *data = read_all(argv[1], &len);

    RvaHandle *handle = rva_open(data, len);
    if (!handle) {
        fprintf(stderr, "rva_open: %s\n", rva_last_error());
        return 1;
    }

    FILE *sizes = fopen(argv[2], "r");
    if (!sizes) {
        perror("sizes fopen");
        return 1;
    }

    int width = 0;
    int height = 0;
    while (fscanf(sizes, "%d %d", &width, &height) == 2) {
        char *json = rva_resolve(handle, (uint32_t)width, (uint32_t)height);
        if (!json) {
            fprintf(stderr, "rva_resolve: %s\n", rva_last_error());
            return 1;
        }
        printf("%d %d %s\n", width, height, json);
        rva_free_string(json);
    }

    fclose(sizes);
    rva_free_handle(handle);
    free(data);
    return 0;
}
