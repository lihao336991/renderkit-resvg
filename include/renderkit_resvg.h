// Copyright 2026 RenderKit contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef RENDERKIT_RESVG_H
#define RENDERKIT_RESVG_H
#include <stdint.h>
#include <stddef.h>
#ifdef __cplusplus
extern "C" {
#endif
// Caller owns all buffers. Output: premultiplied RGBA, width * height * 4.
// 0 success; 1 invalid argument/budget; 2 unsupported/invalid SVG; 3 panic.
uint32_t rk_resvg_abi_version(void);
int32_t rk_resvg_render(const uint8_t *source, size_t source_len,
                       const uint8_t *font, size_t font_len,
                       uint32_t width, uint32_t height,
                       uint8_t *output, size_t output_len);
#ifdef __cplusplus
}
#endif
#endif
