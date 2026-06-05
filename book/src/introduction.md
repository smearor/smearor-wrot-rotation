# Introduction

Welcome to the documentation for `smearor-wrot-rotation`.

This library provides a custom, hardware-accelerated GTK4 container widget—`RotationWidget`—capable of smoothly rotating any child GTK4 widget.

## Why smearor-wrot-rotation?

Rotating elements in modern desktop managers or complex graphical overlays is notoriously difficult, particularly because of input event routing and window boundary management. The `RotationWidget` solves these problems by providing:

1. **Custom GTK4 Layout Subclassing**: Integrates seamlessly with the GTK4 size measurement and layout allocation pipeline.
2. **Reverse Coordinate Mapping**: Maps external pointer (mouse and touch) events to rotated coordinates so the child can be fully interactive.
3. **Transparent Click-Through Regions**: Dynamically sets exact physical bounds onto the native window surface, letting mouse events pass through empty space.
