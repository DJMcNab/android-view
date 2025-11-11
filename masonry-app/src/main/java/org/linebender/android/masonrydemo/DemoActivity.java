package org.linebender.android.masonrydemo;

import android.app.Activity;
import android.os.Bundle;
import android.system.ErrnoException;
import android.system.Os;
import android.view.View;
import android.widget.FrameLayout;

public final class DemoActivity extends Activity {
    static {
        try {
            Os.setenv("RUST_BACKTRACE", "full", false);
            Os.setenv("RUST_LOG", "debug,naga=warn,wgpu_core=warn", false);
            Os.setenv("WGPU_DISCARD_HAL_LABELS", "1", false);
        } catch (ErrnoException e) {
            throw new RuntimeException(e);
        }
         System.loadLibrary("main");
    }

    @Override
    public void onCreate(Bundle state) {
        super.onCreate(state);
        View view = new DemoView(this);
        view.setLayoutParams(
                new FrameLayout.LayoutParams(
                        FrameLayout.LayoutParams.MATCH_PARENT,
                        FrameLayout.LayoutParams.MATCH_PARENT));
        view.setFocusable(true);
        view.setFocusableInTouchMode(true);
        FrameLayout layout = new FrameLayout(this);
        layout.addView(view);
        setContentView(layout);
        view.requestFocus();
    }
}
