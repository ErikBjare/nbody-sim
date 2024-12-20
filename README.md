# N-Body Simulator in Rust

[![built using gptme](https://img.shields.io/badge/built%20using-gptme%20%F0%9F%A4%96-5151f5?style=flat)](https://github.com/ErikBjare/gptme)

A 4K-resolution N-body gravitational simulation built in Rust. This project was created as a modern take on the classic n-body simulations I used to write as programming exercises in the early 2010s.

## About

I've always found n-body simulations fascinating - they were one of my go-to projects for learning new programming languages and exploring optimization techniques. This version was created in a single session using gptme (my AI pair programming tool) to see how quickly we could recreate this classic exercise with modern tools and techniques.

The simulation features:
- 4K resolution rendering using minifb
- Parallel force calculations using rayon
- Orbital system with a central mass
- Mass-based coloring and size scaling
- Glow effects for the central body
- Interactive controls for simulation speed and gravity strength

## Key Features

- **Physics**: Uses basic Newtonian gravity with softening to prevent numerical instability
- **Performance**: Parallel processing for force calculations
- **Visualization**: 
  - Bodies colored by mass (blue=small, red=large)
  - Central star with glow effect
  - Smooth rendering with automatic substeps
- **Controls**:
  - +/- : Adjust simulation speed
  - 1/2 : Adjust gravity strength
  - ESC : Exit

## Building and Running

### Requirements

- Rust and Cargo
- X11 development libraries (on Linux)
  - On Ubuntu/Debian: `sudo apt install xorg-dev`
  - On Arch Linux: `sudo pacman -S libx11`
- A display capable of 4K resolution (3840x2160) or the code can be modified for lower resolutions

### Building

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt install xorg-dev

# Build and run
cargo run --release
```

The release build is strongly recommended for optimal performance, as the simulation is computationally intensive.

### Performance Notes

The simulation is optimized for modern multi-core processors. It uses Rayon for parallel force calculations and can benefit significantly from:
- Multiple CPU cores
- Fast memory
- Modern GPU for window management

## Technical Notes

The simulation uses several optimizations and techniques:
- Parallel force calculations using rayon
- Fixed timestep with variable substeps for smooth animation
- Softening factor to prevent numerical instability
- Efficient circle drawing with glow effects
- Automatic orbital velocity calculations for initial conditions

## Implementation

Built rapidly using gptme, demonstrating how modern AI tools can help quickly prototype complex simulations. The core simulation was implemented in under an hour, with most of the time spent on visual polish and optimization.

## License

MIT
