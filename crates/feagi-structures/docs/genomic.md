# Genomic Structures
This module contains types which describe the structure of a FEAGI Genome

## Cortical Type
A nested enum that can be used to describe the type of cortical area, and the subtype (
in the cases of Core, Input, and Output cortical areas as there are a set number of types of those). Has plenty of helper methods to check restrictions and details of various types of cortical areas.

## Cortical ID
A unique identifier for a cortical area in a genome. Consists of 6 ASCII AlphaNumeric characters. In addition, they also follow the following formatting rules:
- The first character describes the type of cortical area represented
  - 'c': Custom cortical area
  - 'm': Memory cortical area
  - '_': Core cortical area
  - 'i': Input (Sensor) cortical area
  - 'o': Output (Motor) cortical area
  - Anything else is invalid for the first character
- For Custom / Memory cortical areas, the next 5 values can be any alphanumeric character
- For Core cortical areas, as there are a set universal number of core areas, they will map to those specifically.
- For Input / Output cortical areas
  - The second, third, and fourth character will map to a specific cortical type
  - The fifth and sixth characters together represent the cortical areas Cortical Grouping Index, in hexadecimal

These generally should not be instantiated directly, and instead be instantiated with one of the many helper functions.

To see an explanation of all possible Input / Output cortical types in FEAGI, please read the FEAGI documentation here (TODO).

This enum can be instantiated with one of its many "constructor" helper functions for ease of use.

## Classifier

A first-class genome object stored under the top-level `classifiers` key, parallel to `brain_regions`. A classifier is not a brain region and is not exportable as a circuit. The record holds assembly properties, referenced inputs (`kernel_area_id`, `class_area_id`, `field_area_id`), and owned internals (`kernel_memory_id`, `class_memory_id`, `scan_twin_id`). Neuroembryogenesis loads this map; deleting an owned internal deletes the classifier and remaining internals; deleting a referenced input clears that slot; mapping add/remove to internals updates the matching input slot.
