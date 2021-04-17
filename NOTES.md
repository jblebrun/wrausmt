### Code flatmap pattern for parsing NTNTNT locals defs

pattern that's not quite working:
```
(0..n).flatmap(|_| {
  self.read()?
  // return iter here
}).collect()
})
```

### Method accepting closure to self.

In the binary parser, I created a [`read_vec` helper](src/format/binary/values.rs#74) to wrap the pattern of reading a vector encoded in the binary format. I pass the method a closure which reads the items of the vector inside of a mapping created by read_vec. Calling read_vec results in a mutable borrow of self, but since we often want to call another self method inside of the closure, this doesn't work; the closure has already borrowed `self`, so we can't call `read_vec`.

I managed to make this work by passing `self` through the closure as a second
argument. Is there a better way?

### Clean, consistent way of representing variable length immutable arrays.

Right now, for many structure fields that store some array of things where the array may be of any size (but won't chagne once it's stored), I'm using a `Box<[T]>`. 

* Would it be more idiomatic to use a `Vec<T>`? I avoided this because `Vec<T>`
  implies potential mutability to me. 

* Would it be nice to define typealias or newtypes for the various array types?
  This may lead to code that's more self-documenting, and perhaps make it
  easier to change the strategy in the future.

### Instruction storage at runtime

Investigate idea of creating structs for instructions + operands, and storing
them as raw bytes in function instantiations. This would require some usage of `unsafe`, but might be a pretty cool strategy, basically sliding a window over the array of bytes that changes shape as it goes.
