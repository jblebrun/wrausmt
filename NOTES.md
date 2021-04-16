Code flatmap pattern for parsing NTNTNT locals defs

pattern that's not quite working:

(0..n).flatmap(|_| {
  self.read()?
  // return iter here
}).collect()


Method accepting closure to self.
