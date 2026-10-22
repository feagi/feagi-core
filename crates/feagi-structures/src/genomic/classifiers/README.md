# Genome Classifiers

First-class genome object stored under the top-level `classifiers` key, parallel to `brain_regions`.

A classifier is not a brain region and is not exportable as a circuit. Schema v5 stores:

- assembly properties (`name`, `parent_region_id`, `coordinates_3d`)
- referenced inputs (`sample_area_ids`, `class_bindings`)
- owned internals (`sample_memory_id`, `class_memory_id`, `error_area_id`)

`kernel_memory_id` is the v4 name of `sample_memory_id`. The v4-to-v5 migrator renames it and allocates `error_area_id` when the older record does not have one.

Sample and class inputs use the `classifier_sample` and `classifier_class` connectivity rules. The sample memory links to the class memory with `associative_memory`. Neuroembryogenesis loads this map. Deleting an owned internal deletes the classifier. Deleting a referenced input clears that slot.
