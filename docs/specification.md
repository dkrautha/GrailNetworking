# XBF Specification

## Types

### 17 Primitives

- Boolean
- U8, U16, U32, U64, U128, U256
- I8, I16, I32, I64, I128, I256
- F32, F64
- Bytes
- String (UTF-8)

### Vector

A homogenous list of values that has a known length.

### Struct

An aggregate type containing a name as well as named fields. A struct **may not** contain duplicate field names. Should a Struct be sent like this anyway, it should be considered malformed and not be constructed on the receiving end.

## Direct Representations

### Boolean

A boolean should be sent as an unsigned 8-bit integer with the value 0 or 1.
However, should an arbitrary U8 be sent a value of 0 should be taken to be
false, and any other value be taken to be true.

### Integers

All integers should be sent in little endian format, with the least significant
bit first.

For signed integers, they should be represented in two's complement format.

### Floating Point Numbers

All floating point numbers should be sent as 32 or 64 bit IEEE 754 floating
point numbers.

### Variable Length Primitives

Strings should be sent as a sequence of bytes that correspond to UTF-8 code
points. They should first send their length as an unsigned 16-bit integer (in
little endian format), followed by the corresponding number of bytes contained
within the string.

Bytes have the same specification as strings, but with the exception that they
do not have to be a valid sequence of UTF-8 code points.

### Vector

Vectors should first include their length as an unsigned 16-bit integer,
followed by the corresponding number of elements. The type contained within a
Vector is **not** sent to the client. That information is carried in the
metadata.

### Struct

Fields of a Struct are sent in sequence in the order they are listed in the
Struct's metadata (more on that later). When a struct is serialized it should
not send any sort of name information (such as its name or field names), how
many fields it has, nor should it send any type information about its fields.
That information is carried in the metadata.

## Metadata Specification

### Primitives

For primitives, the metadata should be sent as a single byte discriminant value.
After receiving one of these discriminant values, the client should read the
following byte(s) and interpret them as the type given by the discriminant.

Here is the current list of primitives and their expected discriminant values:

| Primitive | Discriminant |
| --------- | ------------ |
| Boolean   | 0            |
| U8        | 1            |
| U16       | 2            |
| U32       | 3            |
| U64       | 4            |
| U128      | 5            |
| U256      | 6            |
| I8        | 7            |
| I16       | 8            |
| I32       | 9            |
| I64       | 10           |
| I128      | 11           |
| I256      | 12           |
| F32       | 13           |
| F64       | 14           |
| Bytes     | 15           |
| String    | 16           |

Strings should always be the final value in the list. The value given to strings
is used by Vectors and Structs to determine what their discriminant value should
be.

### Vector

A discriminant value should first be sent, similarly to primitives (following
the same size requirement). This discriminant value should be 1 greater than
that of the discriminant value for Strings.

Following this, metadata information for the internal type contained within the
Vector will be sent. This process may continue recursively with nested types of
Vectors and Structs.

The length of a vector or the data contained within the vector must **not** be
sent.

### Struct

A discriminant value should first be sent, similarly to primitives (following
the same size requirement). This discriminant value should be 1 greater than
that of the discriminant value for Vectors.

Following this, the name of the Struct should be sent, using the same format as
primitive strings are sent (U16 length and then the bytes). Next, send the
number of fields contained within the Struct as a U16, the same as all other
lengths. Finally, the fields of the Struct should be sent, first the name of the
field as a String, then immediately after the metadata for the type of the
field. This process may continue recursively with nested types of Structs or
Vectors. These name and type pairs will be sent until there are no more fields
left in the Struct.
