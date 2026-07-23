# Infinite Canvas 2D Workspace Paradigm

## Vision
Ermete OS is evolving beyond the archaic concept of discrete, compartmentalized virtual desktops. Instead, we are adopting the **Infinite Canvas 2D Workspace Paradigm**, an unbounded, scrollable, and zoomable 2D plane.

## Core Concepts
1. **Niri as the Foundation**: We leverage Niri's infinite scrolling layout, which already provides a seamless 1D (horizontal) strip of windows.
2. **Expansion to 2D**: The horizontal strip is expanded into a full 2D grid/plane. Windows and groups of windows can be placed anywhere on this X/Y coordinate system.
3. **Vector Zooming**: Navigating the canvas isn't just about panning; it involves semantic vector zooming. Zooming out reveals the "Crow's View" of all open applications, grouped contextually. Zooming in focuses on a specific application or document.

## Architecture
- **Compositor Extensions**: Extending the core compositor to handle an arbitrary 2D coordinate space for window placement rather than fixed outputs or virtual workspaces.
- **Rendering**: Utilizing GPU-accelerated rendering to ensure smooth scaling and panning across the canvas.
- **Input Handling**: Fluid integration with trackpad gestures (pinch-to-zoom, two-finger pan) and keyboard shortcuts for spatial navigation (e.g., jump to a specific context node).

## User Experience
- **Spatial Memory**: Users map tasks to physical locations on their canvas, leveraging human spatial memory instead of abstract workspace numbers.
- **Context Clusters**: Applications related to the same task (e.g., an IDE, a browser, and a terminal) can be clustered together. Zooming out treats the cluster as a single node on the macro-canvas.
