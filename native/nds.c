// Adaptador de los recursos usados por Digraph-dsi. Las estructuras del SDK
// (PrintConsole, touchPosition y FILE) permanecen en C, con su ABI real.
#include <nds.h>
#include <malloc.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static PrintConsole top_console;
static PrintConsole bottom_console;
static int top_bg;
static int bottom_bg;
static int video_claimed;
static int video_visible;
static int counter_ready;

int devy_video_init(void)
{
    if (video_claimed)
        return 0;
    video_claimed = 1;
    powerOn(POWER_ALL_2D);
    lcdMainOnTop();
    videoSetMode(MODE_5_2D);
    videoSetModeSub(MODE_5_2D);
    vramSetBankA(VRAM_A_MAIN_BG);
    vramSetBankB(VRAM_B_MAIN_BG_0x06020000);
    vramSetBankC(VRAM_C_SUB_BG);
    consoleInit(&top_console, 0, BgType_Text4bpp, BgSize_T_256x256,
                22, 3, true, true);
    bgHide(top_console.bgId);
    top_bg = bgInitHidden(3, BgType_Bmp16, BgSize_B16_256x256, 8, 0);
    bottom_bg = bgInitHiddenSub(3, BgType_Bmp8, BgSize_B8_256x256, 0, 0);
    bgSetPriority(top_bg, 3);
    bgSetPriority(bottom_bg, 3);
    setBrightness(3, 0);
    return 1;
}

uint16_t *devy_framebuffer(uint32_t screen)
{
    return bgGetGfxPtr(screen == 0 ? top_bg : bottom_bg);
}

void devy_palette(uint32_t index, uint16_t color) { BG_PALETTE_SUB[index] = color; }

void devy_video_show(void)
{
    if (!video_visible) {
        bgShow(top_bg);
        bgShow(bottom_bg);
        video_visible = 1;
    }
    bgUpdate();
}

void devy_brightness(int top, int bottom)
{
    setBrightness(1, top);
    setBrightness(2, bottom);
}

void devy_console_visible(int visible)
{
    if (visible) bgShow(top_console.bgId);
    else bgHide(top_console.bgId);
}

void devy_console_write(const char *text)
{
    consoleSelect(&top_console);
    fputs(text, stdout);
}

void devy_console_clear(void) { consoleSelect(&top_console); consoleClear(); }

void devy_input(uint32_t *held, uint16_t *x, uint16_t *y)
{
    scanKeys();
    *held = keysHeld();
    if (*held & KEY_TOUCH) {
        touchPosition pos;
        touchRead(&pos);
        *x = pos.px;
        *y = pos.py;
    }
}

uint64_t devy_ticks(void)
{
    if (!counter_ready) { systemCounterSetup(); counter_ready = 1; }
    return systemCounterGetTicks();
}

int devy_outside_vblank(void) { return REG_VCOUNT < 192; }
void devy_wait_vblank(void) { swiWaitForVBlank(); }
int devy_enter_critical(void) { return enterCriticalSection(); }
void devy_leave_critical(int previous) { leaveCriticalSection(previous); }
void *devy_alloc(size_t align, size_t size) { return memalign(align, size); }
void devy_free(void *ptr) { free(ptr); }

void devy_panic(void)
{
    // Funciona incluso si un panic ocurrió antes de inicializar VideoPlugin.
    powerOn(POWER_ALL_2D);
    setBrightness(3, 0);
    videoSetMode(MODE_0_2D);
    videoSetModeSub(MODE_0_2D);
    BG_PALETTE[0] = RGB15(31, 0, 0);
    BG_PALETTE_SUB[0] = RGB15(31, 0, 0);
}

#ifdef DEVY_STORAGE
#include <fat.h>
#include <errno.h>
#include <sys/stat.h>

int devy_storage_init(void) { return fatInitDefault() ? 0 : -1; }

int devy_storage_read(const char *path, uint8_t *buffer, size_t capacity, size_t *count)
{
    FILE *file = fopen(path, "rb");
    if (!file) return -2;
    int result = 0;
    long length = -1;
    if (fseek(file, 0, SEEK_END) == 0) length = ftell(file);
    if (length < 0) result = -2;
    else if ((size_t)length > capacity) result = -3;
    else if (fseek(file, 0, SEEK_SET) != 0) result = -2;
    else {
        *count = fread(buffer, 1, (size_t)length, file);
        if (*count != (size_t)length || ferror(file)) result = -2;
    }
    if (fclose(file) != 0) result = -2;
    return result;
}

int devy_storage_write(const char *path, const uint8_t *buffer, size_t count)
{
    FILE *file = fopen(path, "wb");
    if (!file) return -2;
    size_t written = fwrite(buffer, 1, count, file);
    int flushed = fflush(file);
    int closed = fclose(file);
    return written == count && flushed == 0 && closed == 0 ? 0 : -2;
}

int devy_storage_mkdir(const char *path)
{
    if (mkdir(path, 0777) == 0) return 0;
    struct stat info;
    if (errno == EEXIST && stat(path, &info) == 0 && S_ISDIR(info.st_mode)) return 0;
    return -2;
}
#endif

int devy_hardware_init(void)
{
    if (video_claimed) return 0;
    video_claimed = 1;
    powerOn(POWER_ALL_2D);
    lcdMainOnTop();
    videoSetMode(MODE_0_2D);
    videoSetModeSub(MODE_0_2D);
    vramSetBankA(VRAM_A_MAIN_BG);
    vramSetBankC(VRAM_C_SUB_BG);
    consoleInit(&top_console, 0, BgType_Text4bpp, BgSize_T_256x256, 22, 3, true, true);
    consoleInit(&bottom_console, 0, BgType_Text4bpp, BgSize_T_256x256, 22, 3, false, true);
    setBrightness(3, 0);
    return 1;
}
PrintConsole *devy_get_console(uint32_t screen) { return screen == 0 ? &top_console : &bottom_console; }
