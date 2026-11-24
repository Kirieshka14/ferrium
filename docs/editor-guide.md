# Ferrium Studio Editor Guide

Launch the editor with:

```bash
./run.sh editor
```

## Viewport & Scene Hierarchy
- **Hybrid 2D / 3D Orbit Camera**: Drag to orbit in 3D mode or pan in 2D mode.
- **Ray3 Picking**: Clicking any 3D mesh or 2D sprite runs `Ray3::intersect_obb` against entity bounds to select it in the Inspector.
- **Scene Persistence**: Scenes serialize cleanly to JSON and round-trip all entity transforms, colors, models, and attached `.fy` scripts.
