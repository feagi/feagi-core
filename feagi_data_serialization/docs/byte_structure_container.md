# Byte Structure Layout

All byte transmissions are held in this global container, which can hold 1 or more sets of serialized data.

### Overall Structure (Uncompressed)


<table border="1" id="section-description-" style="border-collapse: collapse; width: 100%;"><colgroup><col style="width: 16.6832%;"></col><col style="width: 16.6832%;"></col><col style="width: 33.3664%;"></col><col style="width: 33.3664%;"></col></colgroup><thead><tr><td>Section Description</td><td>Number of Bytes</td><td>Data Type</td><td>Description</td></tr></thead><tbody><tr><td>Global Header</td><td>  
</td><td></td><td>ALL Byte structures start with these bytes</td></tr><tr><td>  
</td><td>1</td><td>u8</td><td>Byte Structure Version</td></tr><tr><td>  
</td><td>2</td><td>u16</td><td>Increment Counter</td></tr><tr><td>  
</td><td>1</td><td>u8</td><td>Number of contained structures</td></tr><tr><td>Structure Lookup Header</td><td>  
</td><td>  
</td><td> These 8 bytes are repeated for every contained structure
</td></tr><tr><td>
</td><td>4</td><td>u32</td><td>First byte index of the structure (relative to this entire byte array)</td></tr><tr>
<td>
</td><td>4</td><td>u32</td><td>Number of bytes to read for the structure from the start index</td></tr><tr><td>Per Structure header</td><td>
</td><td>  
</td><td> Every struct in follows in sequence. They always start with 2 per header bytes as described below, then the rest is the data.
</td></tr><tr><td>  
</td><td>1</td><td>u8</td><td>Structure type identifier</td></tr><tr><td>  
</td><td>1</td><td>u8</td><td>Structure Version identifier</td></tr><tr><td>  
</td><td>?</td><td>any</td><td>the data of the struct</td></tr></tbody></table>

#### Specific Implementations
The following types of data are possible serializable types that can be contained by the Feagi Byte Container:
- [JSON](specific_structures/001%20-%20JSON.md)
- [Neuron Potential Data (XYZP)](specific_structures/011%20-%20Neuron%20Potential%20Data%20(XYZP).md)


#### Version
As of time of writing, the current version is "2"

#### Note on Compression

We use the [Deflate](https://en.wikipedia.org/wiki/Deflate) compression before sending / reading this data over the network to cut down bandwidth costs, and because this is a fast and built in method for many languages and interfaces.