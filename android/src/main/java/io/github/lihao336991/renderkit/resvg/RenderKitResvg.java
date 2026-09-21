// Copyright 2026 RenderKit contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
package io.github.lihao336991.renderkit.resvg;

/** Fail-closed Java binding for the bundled resvg JNI library. */
public final class RenderKitResvg {
    /** Native ABI exposed by this release. */
    public static final int ABI_VERSION = 1;
    private static final boolean AVAILABLE;

    static {
        boolean available = false;
        try {
            System.loadLibrary("renderkit_resvg_jni");
            available = abiVersionNative() == ABI_VERSION;
        } catch (LinkageError | SecurityException ignored) {
            // Absence is a supported capability state for optional consumers.
        }
        AVAILABLE = available;
    }

    private RenderKitResvg() {}

    /**
     * Returns whether the JNI library loaded and reported the expected ABI.
     *
     * @return {@code true} only when the native binding is usable
     */
    public static boolean isAvailable() {
        return AVAILABLE;
    }

    /**
     * Renders SVG and optional font bytes into straight ARGB pixels.
     *
     * @param source UTF-8 SVG source
     * @param font optional font bytes, or {@code null}
     * @param width requested output width in pixels
     * @param height requested output height in pixels
     * @return straight ARGB pixels, or {@code null} when validation or rendering fails
     */
    public static int[] render(byte[] source, byte[] font, int width, int height) {
        if (!AVAILABLE) return null;
        return renderNative(source, font, width, height);
    }

    private static native int abiVersionNative();
    private static native int[] renderNative(byte[] source, byte[] font, int width, int height);
}
