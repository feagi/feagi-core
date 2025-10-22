# Neuron Voxel Structures
This module contains types for representing neural activity in spatial voxel form within FEAGI.

## Overview
In FEAGI's visualization (such as Brain Visualizer), neurons are represented as voxels where each voxel can contain one or more neurons at a specific 3D spatial location. A voxel stores its position coordinates (x, y, z) and its activation potential (p), hence the "XYZP" format. These structures provide efficient ways to store, transmit, and manipulate neural activity data across cortical areas.

## NeuronVoxelXYZP
A single neuron voxel storing spatial coordinates and activation potential in XYZP format.

Each voxel represents neural activity at a specific 3D location within a cortical area, along with its current activation/voltage level.

**Key Properties:**
- **cortical_coordinate**: The (x, y, z) position within the cortical area (uses Cartesian coordinate system)
- **potential**: The activation/voltage value of the neuron (stored as f32). In the case of sensors / motors, these values typically range from  -1 to 1.


## NeuronVoxelXYZPArrays
Essentially an array of NeuronVoxelXYZPs with some helper functions and internal optimizations.
This is used per cortical area to represent firing neurons it has.

## CorticalMappedXYZPNeuronVoxels
Essentially a Dictionary (Hashmap) key'd by CorticalID, to the corresponding NeuronVoxelXYZPArray.
This structure is used to represent the firings of neurons across the genome.
It also supports serialization via the FeagiByteContainer.


