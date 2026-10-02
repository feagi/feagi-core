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

The class output is named `{classifier name} class output`. A second field includes
that field's name.

Kernel mode writes one class for the whole field. The field must match the kernel
area, and the output is `1×1×n`, the same shape as the class input. Depth `z` fires
for class `z`. `mp_driven_psp` stays off.

Scanner mode writes a location map. The output is `field_w × field_h × 1`
(`detection_twin_shape`). Each detected pixel fires once, with the winning class
(highest summed vote, ties to the lower id) as its exact potential. `mp_driven_psp`
is on, so a 1:1 mapping from that output (weight 1, uniform PSP, target threshold at
or below `1 / class_count`, no leak or accumulation) hands the class value to an OPU
unchanged. Encode and decode through `neuron_voxels::class_potential`.

Neuroembryogenesis loads this map onto the connectome. Area delete and mapping edits update the record and the mappings it requires.

The record is authoritative for its own edges (`Classifier::required_mappings`). When the
kernel area is also a field, its edge into kernel memory carries two rules: `episodic_memory`
(encode) and `episodic_scan` (scan). Classifier endpoints add or remove one rule without
dropping the other, and genome load adds any required rule a saved mapping list lacks. Rules
are built by `classifier_mapping_rule`.
