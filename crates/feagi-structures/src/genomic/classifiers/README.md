# Genome Classifiers

First-class genome object stored under the top-level `classifiers` key, parallel to `brain_regions`.

A classifier is not a brain region and is not exportable as a circuit. It records:

- assembly properties (`name`, `parent_region_id`, `coordinates_3d`)
- referenced inputs (`kernel_area_id`, `class_area_id`, `field_area_id`)
- owned internals (`kernel_memory_id`, `class_memory_id`, `scan_twin_id`)

Neuroembryogenesis loads this map onto the connectome. Area delete and mapping edits update the record and the mappings it requires.
