#define _GNU_SOURCE
#include <assert.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>
#include <dlfcn.h>

static const char *forwarded;
static int forwarded_length;
static void fake_pango(void *layout, const char *text, int length) {
    (void)layout;
    forwarded = text;
    forwarded_length = length;
}
static void *fake_dlsym(void *handle, const char *symbol) {
    (void)handle;
    assert(strcmp(symbol, "pango_layout_set_text") == 0);
    return (void *)fake_pango;
}
#define dlsym fake_dlsym
#include "../libsolar_brand.c"

int main(void) {
    size_t page = (size_t)sysconf(_SC_PAGESIZE);
    char *memory = mmap(NULL, page * 2, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    assert(memory != MAP_FAILED);
    assert(mprotect(memory + page, page, PROT_NONE) == 0);
    char *span = memory + page - 4;
    memcpy(span, "Niri", 4); /* no NUL; next byte is inaccessible */
    pango_layout_set_text(NULL, span, 4);
    assert(strcmp(forwarded, "SolarUI WM Engine") == 0);
    memcpy(span, "text", 4);
    pango_layout_set_text(NULL, span, 4);
    assert(forwarded == span && forwarded_length == 4);
    pango_layout_set_text(NULL, span, 0);
    assert(forwarded == span && forwarded_length == 0);
    pango_layout_set_text(NULL, NULL, 0);
    assert(forwarded == NULL);
    assert(munmap(memory, page * 2) == 0);
    return 0;
}
