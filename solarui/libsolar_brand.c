#define _GNU_SOURCE
#include <dlfcn.h>
#include <pthread.h>
#include <stddef.h>
#include <string.h>
#include <strings.h>

typedef void PangoLayout;
static void (*real_pango_layout_set_text)(PangoLayout *, const char *, int);
static pthread_once_t resolve_once = PTHREAD_ONCE_INIT;

static void resolve_pango(void) {
    real_pango_layout_set_text = dlsym(RTLD_NEXT, "pango_layout_set_text");
}

static int text_matches(const char *text, size_t size, const char *label, int ignore_case) {
    size_t label_size = strlen(label);
    return size == label_size && (ignore_case ? strncasecmp(text, label, size) : memcmp(text, label, size)) == 0;
}

/* Optional shell-only branding. Never intercept GTK minimize: a GTK object is
 * not a compositor window ID, and the focused window may belong to another app. */
void pango_layout_set_text(PangoLayout *layout, const char *text, int length) {
    pthread_once(&resolve_once, resolve_pango);
    if (!real_pango_layout_set_text) return;
    if (text) {
        /* Pango accepts a byte span without a trailing NUL when length >= 0. */
        size_t size = length < 0 ? strlen(text) : strnlen(text, (size_t)length);
        const char *replacement = NULL;
        if (text_matches(text, size, "Niri", 1) || text_matches(text, size, "niri compositor", 1))
            replacement = "SolarUI WM Engine";
        else if (text_matches(text, size, "Çubuk: taskbar", 0) || text_matches(text, size, "Bar: taskbar", 0))
            replacement = "Çubuk: Görev Çubuğu";
        else if (text_matches(text, size, "Çubuk: default", 0) || text_matches(text, size, "Bar: default", 0))
            replacement = "Çubuk: Üst Çubuk (Noctalia)";
        if (replacement) {
            real_pango_layout_set_text(layout, replacement, -1);
            return;
        }
    }
    real_pango_layout_set_text(layout, text, length);
}
