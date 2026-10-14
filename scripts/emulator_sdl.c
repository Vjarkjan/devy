// Sólo para la presentación del emulador en Xvfb. No se enlaza con la ROM.
#include <dlfcn.h>
#include <stdint.h>

void *SDL_CreateWindow(const char *title, int x, int y, int w, int h, uint32_t flags)
{
    typedef void *(*Create)(const char *, int, int, int, int, uint32_t);
    Create create = (Create)dlsym(RTLD_NEXT, "SDL_CreateWindow");
    return create(title, x, y, w, h, flags & ~UINT32_C(2));
}

void *SDL_CreateRenderer(void *window, int index, uint32_t flags)
{
    (void)index;
    (void)flags;
    typedef void *(*Create)(void *, int, uint32_t);
    Create create = (Create)dlsym(RTLD_NEXT, "SDL_CreateRenderer");
    return create(window, -1, UINT32_C(1));
}
