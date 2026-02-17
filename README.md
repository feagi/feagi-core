# FEAGI Core

Framework for Evolutionary Artificial General Intelligence - High-performance 
Rust libraries shared by various FEAGI applications and runtimes.

## What is FEAGI?

FEAGI (Framework for Evolutionary Artificial General Intelligence) is a bio-inspired neural architecture that models brain structures and dynamics. FEAGI Core provides the foundational Rust libraries for building neural networks that learn and adapt like biological brains.

Unlike traditional neural networks, FEAGI:
- Models individual neurons with realistic dynamics (membrane potential, leak, refractory periods)
- Supports heterogeneous brain regions with distinct properties
- Enables structural plasticity (neurogenesis, synaptogenesis)
- Runs in real-time with spike-based computation
- Scales from microcontrollers to servers

## Included Crates

FEAGI Core is organized as a workspace of focused crates:

### feagi-agent
- Server client interfaces for interactions between FEAGI agents and the FEAGI server

TODO the rest description



## Feature Flags

There are various feature flags for selecting feature sets and target platforms

TODO description

## Community and Support

- **[Discord](https://discord.gg/PTVC8fyGN8)**
- **[Website](https://neuraville.com/feagi)**

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

Copyright 2025 Neuraville Inc.

## Citation

If you use FEAGI in your research, please cite:

```bibtex
@article{nadji2020brain,
  title={A brain-inspired framework for evolutionary artificial general intelligence},
  author={Nadji-Tehrani, Mohammad and Eslami, Ali},
  journal={IEEE transactions on neural networks and learning systems},
  volume={31},
  number={12},
  pages={5257--5271},
  year={2020},
  publisher={IEEE}
}
```

---
