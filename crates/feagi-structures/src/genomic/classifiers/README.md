# Genome Classifiers

First-class genome object stored under the top-level `classifiers` key, parallel to `brain_regions`.

A classifier is not a brain region and is not exportable as a circuit. It records:

- assembly properties (`name`, `parent_region_id`, `coordinates_3d`)
- referenced inputs (`kernel_area_id`, `class_area_id`, `field_area_id`)

- owned internals (`kernel_memory_id`, `class_memory_id`, `scan_twin_id`)

## Class maps are one layer

Kernel-mode `class_area_id` must be `1×1×n`: depth index `z` is class `z`.

Scanner mode stores an explicit `class_count` (required; `1..=9999`). The scanner mask is
`field_w × field_h × 1`; each labeled pixel's potential is `(class_id + 1) / class_count`
and zero means unlabeled.

Every detection twin is `field_w × field_h × 1` in both modes (`detection_twin_shape`).
The classifier writes each detected pixel once, with the winning class (highest summed vote,
ties to the lower id) as its exact potential, using a force-fire that bypasses threshold and
leak. Twins are created with `mp_driven_psp` on, so a 1:1 mapping from a twin (weight 1,
uniform PSP, target threshold at or below `1 / class_count`, no leak or accumulation) hands
the class value to an OPU unchanged. Encode and decode through
`neuron_voxels::class_potential`.

A genome that needs one neuron per class downstream expands the twin itself: a mapping into
a `W×H×C` area whose firing threshold increments along Z.

Neuroembryogenesis loads this map onto the connectome. Area delete and mapping edits update the record and the mappings it requires.
