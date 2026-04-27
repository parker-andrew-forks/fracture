# flatpak-launcher

This is to wrap around the Fracture tool to catch unrecoverable errors thrown by pipewire, the event loop, WebGPU, or GTK. It works within a flatpak container. If it catches a panic, it starts again with the safest possible options.
