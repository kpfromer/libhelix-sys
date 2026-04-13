/*
 * helix_shim.c — Static-init shim for libhelix-mp3
 *
 * Replaces MP3InitDecoder/MP3FreeDecoder with a zero-allocation path.
 * All decoder state is placed into a caller-provided buffer using
 * arena-style sub-allocation.
 */

#include <string.h>
#include <stddef.h>
#include <stdint.h>
#include "coder.h"  /* includes mp3common.h -> mp3dec.h */

/* Align offset up to `align` bytes (must be power of 2) */
#define ALIGN_UP(offset, align) (((offset) + (align) - 1) & ~((align) - 1))

/*
 * Compute the total arena size needed for MP3DecInfo + all sub-structs.
 * Each sub-struct is aligned to 8 bytes for safety on all platforms.
 */
#define ARENA_ALIGN 8

const size_t HELIX_MP3_DECODER_SIZE =
    ALIGN_UP(sizeof(MP3DecInfo), ARENA_ALIGN) +
    ALIGN_UP(sizeof(FrameHeader), ARENA_ALIGN) +
    ALIGN_UP(sizeof(SideInfo), ARENA_ALIGN) +
    ALIGN_UP(sizeof(ScaleFactorInfo), ARENA_ALIGN) +
    ALIGN_UP(sizeof(HuffmanInfo), ARENA_ALIGN) +
    ALIGN_UP(sizeof(DequantInfo), ARENA_ALIGN) +
    ALIGN_UP(sizeof(IMDCTInfo), ARENA_ALIGN) +
    ALIGN_UP(sizeof(SubbandInfo), ARENA_ALIGN);

/*
 * Initialize the MP3 decoder into a caller-provided buffer.
 *
 * `buf` must be at least HELIX_MP3_DECODER_SIZE bytes and aligned to 8 bytes.
 * Returns the decoder handle (same pointer as buf), or NULL if buffer too small.
 *
 * This completely replaces MP3InitDecoder() — no malloc, no free.
 */
HMP3Decoder helix_mp3_init_into(void *buf, size_t buf_len) {
    if (!buf || buf_len < HELIX_MP3_DECODER_SIZE)
        return (HMP3Decoder)0;

    /* Zero the entire arena */
    memset(buf, 0, HELIX_MP3_DECODER_SIZE);

    uint8_t *arena = (uint8_t *)buf;
    size_t offset = 0;

    /* MP3DecInfo sits at the start */
    MP3DecInfo *mp3DecInfo = (MP3DecInfo *)(arena + offset);
    offset += ALIGN_UP(sizeof(MP3DecInfo), ARENA_ALIGN);

    /* Sub-structs follow contiguously */
    mp3DecInfo->FrameHeaderPS = (void *)(arena + offset);
    offset += ALIGN_UP(sizeof(FrameHeader), ARENA_ALIGN);

    mp3DecInfo->SideInfoPS = (void *)(arena + offset);
    offset += ALIGN_UP(sizeof(SideInfo), ARENA_ALIGN);

    mp3DecInfo->ScaleFactorInfoPS = (void *)(arena + offset);
    offset += ALIGN_UP(sizeof(ScaleFactorInfo), ARENA_ALIGN);

    mp3DecInfo->HuffmanInfoPS = (void *)(arena + offset);
    offset += ALIGN_UP(sizeof(HuffmanInfo), ARENA_ALIGN);

    mp3DecInfo->DequantInfoPS = (void *)(arena + offset);
    offset += ALIGN_UP(sizeof(DequantInfo), ARENA_ALIGN);

    mp3DecInfo->IMDCTInfoPS = (void *)(arena + offset);
    offset += ALIGN_UP(sizeof(IMDCTInfo), ARENA_ALIGN);

    mp3DecInfo->SubbandInfoPS = (void *)(arena + offset);
    /* offset += ALIGN_UP(sizeof(SubbandInfo), ARENA_ALIGN); */

    return (HMP3Decoder)mp3DecInfo;
}

/*
 * Provide AllocateBuffers/FreeBuffers stubs so we can drop buffers.c
 * (which pulls in malloc/free). These are referenced by MP3InitDecoder
 * and MP3FreeDecoder in mp3dec.c, but our Rust wrapper never calls those.
 * The stubs exist solely to satisfy the linker.
 */
MP3DecInfo *AllocateBuffers(void) {
    return (MP3DecInfo *)0;
}

void FreeBuffers(MP3DecInfo *mp3DecInfo) {
    (void)mp3DecInfo;
}
