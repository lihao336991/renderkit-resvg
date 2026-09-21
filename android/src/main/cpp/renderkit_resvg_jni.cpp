// Copyright 2026 RenderKit contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include <jni.h>
#include <cstdint>
#include <new>
#include <vector>
#include "renderkit_resvg.h"

extern "C" JNIEXPORT jint JNICALL
Java_io_github_lihao336991_renderkit_resvg_RenderKitResvg_abiVersionNative(JNIEnv *, jclass) {
  return rk_resvg_abi_version();
}

extern "C" JNIEXPORT jintArray JNICALL
Java_io_github_lihao336991_renderkit_resvg_RenderKitResvg_renderNative(
    JNIEnv *env, jclass, jbyteArray source, jbyteArray font, jint width, jint height) {
  if (!source || width <= 0 || height <= 0 || width > 2048 || height > 2048) return nullptr;
  const int64_t pixel_count = static_cast<int64_t>(width) * height;
  if (pixel_count > 3 * 1024 * 1024) return nullptr;
  try {
    const jsize length = env->GetArrayLength(source);
    const jsize font_length = font ? env->GetArrayLength(font) : 0;
    if (env->ExceptionCheck() || length > 256 * 1024 || font_length > 16 * 1024 * 1024) return nullptr;
    std::vector<uint8_t> bytes(length), fonts(font_length), rgba(static_cast<size_t>(pixel_count) * 4);
    env->GetByteArrayRegion(source, 0, length, reinterpret_cast<jbyte *>(bytes.data()));
    if (font_length) env->GetByteArrayRegion(font, 0, font_length, reinterpret_cast<jbyte *>(fonts.data()));
    if (env->ExceptionCheck()) return nullptr;
    if (rk_resvg_render(bytes.data(), bytes.size(), fonts.data(), fonts.size(), width, height,
                        rgba.data(), rgba.size()) != 0) return nullptr;
    std::vector<jint> argb(static_cast<size_t>(pixel_count));
    for (size_t i = 0; i < argb.size(); ++i) {
      const uint32_t alpha = rgba[i * 4 + 3];
      auto channel = [alpha](uint8_t value) -> uint32_t {
        return alpha ? (uint32_t(value) * 255 + alpha / 2) / alpha : 0;
      };
      argb[i] = (alpha << 24) | (channel(rgba[i * 4]) << 16) |
                (channel(rgba[i * 4 + 1]) << 8) | channel(rgba[i * 4 + 2]);
    }
    auto result = env->NewIntArray(static_cast<jsize>(argb.size()));
    if (!result || env->ExceptionCheck()) return nullptr;
    env->SetIntArrayRegion(result, 0, static_cast<jsize>(argb.size()), argb.data());
    return env->ExceptionCheck() ? nullptr : result;
  } catch (const std::bad_alloc &) {
    return nullptr;
  } catch (...) {
    return nullptr;
  }
}
