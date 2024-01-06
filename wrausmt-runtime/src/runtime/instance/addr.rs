use {
    super::{
        DataInstance, ElemInstance, FunctionInstance, GlobalInstance, MemInstance, TableInstance,
    },
    std::marker::PhantomData,
    wrausmt_common::marker,
};

/// Function instances, table instances, memory instances, and global instances,
/// element instances, and data instances in the store are referenced with
/// abstract addresses. These are simply indices into the respective store
/// component. In addition, an embedder may supply an uninterpreted set of host
/// addresses.
///
/// This is a type-safe wrapper around a u32 to use for addressing in the
/// runtime.
///
/// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#addresses
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Address<T: AddressType>(pub u32, PhantomData<T>);
impl<T: AddressType> Address<T> {
    pub fn new(value: u32) -> Self {
        Self(value, PhantomData)
    }
}

/// Instances that have an address should implement this trait, and specify
/// [`AddressType`]. There's a macro to help with this.
pub trait Addressable {
    type AddressType: AddressType;
}

/// A marker trait for making addresses type-safe.
pub trait AddressType: Copy {}

/// Use `addressable!(InstanceType, AddressType)` to set up a new address for
/// an instance type. It creates the marker trait for the [`Address`] and
/// implements  `Addressable` using the new addresst type.
macro_rules! addressable {
    ($t:ty, $a:ident) => {
        marker!($a: AddressType);
        impl Addressable for $t {
            type AddressType = $a;
        }
    };
}

impl<A: AddressType> From<u32> for Address<A> {
    fn from(value: u32) -> Self {
        Address::new(value)
    }
}

addressable!(FunctionInstance, Function);
addressable!(TableInstance, Table);
addressable!(DataInstance, Data);
addressable!(MemInstance, Memory);
addressable!(GlobalInstance, Global);
addressable!(ElemInstance, Elem);

/// A contiguous range of [`Address`].
///
/// Note: It is possible to remove this and use
/// [`std::ops::Range<Address<T>>`] if the step trait unstable feature is
/// enabled.
pub struct AddressRange<A: AddressType> {
    pub start: Address<A>,
    pub end:   Address<A>,
}

impl<A: AddressType> AddressRange<A> {
    pub fn new(start: impl Into<Address<A>>, end: impl Into<Address<A>>) -> Self {
        AddressRange {
            start: start.into(),
            end:   end.into(),
        }
    }
}

impl<A: AddressType> IntoIterator for AddressRange<A> {
    type IntoIter = AddressRangeIntoIter<A>;
    type Item = Address<A>;

    fn into_iter(self) -> Self::IntoIter {
        AddressRangeIntoIter {
            cur:  self.start,
            last: self.end,
        }
    }
}

pub struct AddressRangeIntoIter<A: AddressType> {
    cur:  Address<A>,
    last: Address<A>,
}

impl<A: AddressType> Iterator for AddressRangeIntoIter<A> {
    type Item = Address<A>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur.0 > self.last.0 {
            None
        } else {
            let out = self.cur;
            self.cur.0 += 1;
            Some(out)
        }
    }
}
