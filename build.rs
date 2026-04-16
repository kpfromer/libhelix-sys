fn main() {
    let mut build = cc::Build::new();

    build
        .files([
            // Core libhelix sources
            "libhelix/src/mp3dec.c",
            "libhelix/src/mp3tabs.c",
            // Platform decode routines
            "libhelix/src/real/bitstream.c",
            // buffers.c replaced by csrc/helix_shim.c (no malloc/free)
            "libhelix/src/real/dct32.c",
            "libhelix/src/real/dequant.c",
            "libhelix/src/real/dqchan.c",
            "libhelix/src/real/huffman.c",
            "libhelix/src/real/hufftabs.c",
            "libhelix/src/real/imdct.c",
            "libhelix/src/real/polyphase.c",
            "libhelix/src/real/scalfact.c",
            "libhelix/src/real/stproc.c",
            "libhelix/src/real/subband.c",
            "libhelix/src/real/trigtabs.c",
            // Our static-init shim
            "csrc/helix_shim.c",
        ])
        .include("libhelix/src/pub")
        .include("libhelix/src/real")
        .warnings(false)
        .warnings(false);

    // -mlongcalls is required for Xtensa targets (ESP32) to avoid
    // "call target out of range" errors when calling memcpy/memset/memmove
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("xtensa") {
        build.flag("-mlongcalls");
    }

    build.compile("helixmp3");
}
