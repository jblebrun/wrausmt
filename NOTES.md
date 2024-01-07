

### Id management

The text format supports named IDs as a convenience. These don't appear in the abstract syntax, but it may be nice to track these at runtime for informational purposes during debugging/error reporting.

### Method accepting closure to self.

In the binary parser, I created a [`read_vec` helper](src/format/binary/values.rs#74) to wrap the pattern of reading a vector encoded in the binary format. I pass the method a closure which reads the items of the vector inside of a mapping created by read_vec. Calling read_vec results in a mutable borrow of self, but since we often want to call another self method inside of the closure, this doesn't work; the closure has already borrowed `self`, so we can't call `read_vec`.

I managed to make this work by passing `self` through the closure as a second argument. 

This seems to be a reasonable way to do this, and it telegraphs the intent nicelyl

### Marker Traits for Type Safety

Marker traits are used to provide 0-overhead (as far as data overhead goes) type safety for a few things:

## Index Resolution

Indexes are loaded during parse time, but because they can refer to items ahead and behind, they aren't resolved until the full module is read in. So for a while, the `Index` data type lives in an unresolved state, but carries the same data that it will if it's validated. To handle this, indices are generic against a `ResolvedState`. So when first read in, they are `Index<Unresolved>`. Later, after being validated, they are returned as `Index<Resolved>`. The only way to get an `Index<Resolved>` (safelyt) is to pass an `Index<Unresolved>` through the proper validator method. And functions that need to accept a valid index only accept `Index<Resolved>`. So this is a cool way to determine, at compile time, that only resolved indices make it to runtime code. (Of course, with unsafe code, anything can happen....).


## Index Address Spaces
Indices and Addresses have "spaces" in WASM. For example, the Index may refer to a function, a memory, a table, etc, and similarly for addresses. However the underlying address type is the same. To differentiate the types at compile time, they're generic over type that specifies the index or address space. Using `PhantomData` items inside the strucutres, we can ensure that functions accept only the proper address type, reducing the likelihood of typos or copy-pasting resulting in allowing the wrong address type to be passed through when working with address and index types.









