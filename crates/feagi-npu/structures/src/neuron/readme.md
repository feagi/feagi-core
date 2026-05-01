# Neuron Module

### (WIP)

This module handles the storing and processing of various neuron models on various different device architectures


### Dimensional vs non-dimensional
Any neuron model / cortical area that has defined XYZ dimensions is defined as dimensional. 
Any cortical area that doesn't (such as memory cortical areas) are non-dimensional

Due to how different connectivity rules interact between dimensional and non-dimensional types, we group
these neuron types together to make logic constant between them.