// Copyright 2026 RenderKit contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
package io.github.lihao336991.renderkit.resvg;

import java.nio.file.Files;
import java.nio.file.Paths;

public final class JniSmoke {
    public static void main(String[] args) throws Exception {
        if (!RenderKitResvg.isAvailable()) throw new AssertionError("binding unavailable");
        byte[] source = Files.readAllBytes(Paths.get(args[0]));
        int[] pixels = RenderKitResvg.render(source, null, 200, 100);
        if (pixels == null || pixels.length != 20_000) throw new AssertionError("render failed");
        boolean markerPixel = false;
        for (int pixel : pixels) if ((pixel >>> 24) != 0) { markerPixel = true; break; }
        if (!markerPixel) throw new AssertionError("marker pixels missing");
        if (RenderKitResvg.render(source, null, 2048, 1537) != null)
            throw new AssertionError("pixel budget accepted");
        if (RenderKitResvg.render("invalid".getBytes("UTF-8"), null, 200, 100) != null)
            throw new AssertionError("invalid XML accepted");
        System.out.println("RenderKitResvg Android JNI smoke passed");
        System.exit(0);
    }
}
