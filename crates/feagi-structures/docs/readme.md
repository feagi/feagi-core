# FEAGI Data Structures

This crate is essentially the root crate that all other crates for FEAGI depends on. It defines the core data 
structures and functions used by all FEAGI libraries and applications. 

There are several main concepts to be aware of:

## Genomic Structures

There are various Genomic structures / identifiers to be aware of

### Cortical Type
CorticalType is a nested enum that differentiates between different types of cortical areas. The possible types are:
- Memory: Cortical areas to store memories from learning
- Custom: In between Cortical Areas that handle the bulk of processing / thinking in a FEAGI model
- Core: Universal Cortical areas that are found in all genomes. This has a sub Enum CoreCorticalType to define these different cortical types
- Sensor: Defines some sensory input to FEAGI to allow it to sense the world, and uses sub enum SensorCorticalType to differentiate the sensor type
- Motor: Defines some motor output of FEAGI to allow world interactions, and uses sub enum MotorCorticalType to differentiate what type of motor it is

Of note, the types of Sensors and Motors possible are defined by the Template Files, which are compiled via Rust Macros into the various SensorCorticalTypes and MotorCorticalTypes.

Effectively, this enum system allows defining a class of cortical area.


### Cortical ID
Cortical IDs are identifiers for a specific cortical area within a genome, essentially unique IDs per area. Functionally, they are defined as 6 ASCII bytes that follow various patterns depending on the type of area they represent. As a user you just need to be aware that there isd a unique ID per genome, but otherwise the system is largely automatic in the background.

### Descriptors

There are certain types of numbers to be aware of as well.

#### Cortical Units
Since you can have multiple cortical areas of the same sensor / motor cortical type, the Cortical Unit Index is an u8 number defining which specific instance of a given cortical type is being referenced.

#### Cortical Channels
A single cortical sensor / motor cortical area can be divided into multiple channels, to represent multiple inputs / outputs of a given type of data.

## Neuron Voxels

As seen in Brain Visualizer, a neuron (or a grouping of neurons) can be visualized as a 3D voxel. This state can be stored and communicated via NeuronVoxelXYZP structures, which define the XYZ coordinate of the neuron relative to the cortical area 0,0,0 root, and the P potential float, which in the case of sensors / motors, is often a float between -1 and 1.

Voxels may be grouped into arrays for a single cortical area in a NeuronVoxelXYZPArray structure, and multiple of these arrays for multiple cortical areas are mapped by their corresponding Cortical ID in a CorticalMappedXYZPNeuronVoxel structure, which acts as a dictionary.

## Templates
The sensor and motor template files are essentially tables used by macros to generate code corresponding to various types of sensors (from infrared sensors, proximity, camera, etc) and motors (servos, actuators, etc).

