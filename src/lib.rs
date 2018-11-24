//! # E-Num(ber)
//!
//! Serialize enums into numbers.
//!
//! ## WARNING
//!
//! This library works with variant fields (e.g. `Variant1(u64)`) by
//! bitshifting the number representation of the contained value over
//! enough so that the tagging can fit on the right of the number. If
//! you're dealing with very large numbers in the fields or have a ton
//! of variants, data on the left side of the value will likely be lost.
//!
//! ## Basic Usage
//!
//! ```
//! #[macro_use]
//! extern crate e_num;
//!
//! use e_num::ENum;
//!
//! #[derive(ENum)]
//! enum A {
//!   B,
//!   C(u64),
//! }
//!
//! fn main() {
//!   let b: usize = A::B.to_num();
//!   println!("b as a number: {:#b}", b);
//!   let b = A::from_num(b);
//!   assert!(match b {
//!     A::B => true,
//!     _ => false,
//!   });
//!   let c = A::C(85).to_num();
//!   println!("c as a number: {:#b}", c);
//!   let c = A::from_num(c);
//!   assert!(match c {
//!     A::C(inner) => {
//!       assert_eq!(inner, 85);
//!       true
//!     }
//!     _ => false,
//!   });
//! }
//! ```
//!
//! ## `start_at` and constant variants
//!
//! ```
//! #[macro_use]
//! extern crate e_num;
//!
//! use e_num::ENum;
//!
//! #[derive(ENum)]
//! // where the non-constant variants will start counting from
//! #[e_num(start_at = 9)]
//! enum A {
//!   // pulls the specified variant out from the rest of them
//!   // and matches it against that number. constant variants
//!   // can't have a field.
//!   #[e_num(constant = 2)]
//!   B,
//!   C,
//!   D,
//!   E,
//! }
//!
//! fn main() {
//!   assert_eq!(A::B.to_num(), 2);
//!   assert_eq!(A::C.to_num(), 9);
//!   assert_eq!(A::D.to_num(), 10);
//!   assert_eq!(A::E.to_num(), 11);
//! }
//! ```

#[allow(unused_imports)]
#[doc(hidden)]
#[macro_use]
extern crate e_num_derive;

#[doc(hidden)]
pub use e_num_derive::*;

#[cfg(test)]
mod tests {
  use ENum;
  #[derive(ENum)]
  #[e_num(start_at = 0)]
  enum Test1 {
    A,
    B(usize),
    C,
  }
  #[test]
  fn basic() {
    assert!(match Test1::from_num(0) {
      Test1::A => true,
      _ => false,
    });
  }

  #[test]
  fn value_from_enum() {
    assert!(match Test1::from_num(0b10101) {
      Test1::B(v) => v == 0b101,
      _ => false,
    });
  }

  #[derive(ENum)]
  enum Test2 {
    #[e_num(constant = 9)]
    A,
    B,
  }

  #[test]
  fn constant_variant() {
    assert!(Test2::A.to_num() == 9);
  }
}

pub trait ENum: Sized {
  /// Parse a number into the type.
  ///
  /// If you're `impl`ing `ENum` yourself, you don't *need* to
  /// define this function, there is a default implementation
  /// that uses the output from `try_from_num()`.
  ///
  /// # Examples
  ///
  /// ```
  /// # #[macro_use] extern crate e_num;
  /// # use e_num::ENum;
  /// # #[derive(ENum)]
  /// # enum A {
  /// #   B,
  /// #   C,
  /// # }
  /// # let num = A::B.to_num();
  /// let a = A::from_num(num);
  /// assert!(match a {
  ///   A::B => true,
  ///   _ => false,
  /// });
  /// ```
  ///
  /// # Panics
  ///
  /// This function should panic if it cannot parse the number into
  /// its type; e.g. you should only pass to this function the output
  /// of `.to_num()`. If you want to handle a parsing error, use `try_from_num()`.
  fn from_num(num: usize) -> Self {
    Self::try_from_num(num).expect("Couldn't parse number into type")
  }
  /// The error-handling counterpart of `from_num()`.
  ///
  /// Use this if you're not sure whether or not the number you're passing to the
  /// function is valid.
  ///
  /// # Examples
  ///
  /// ```
  /// # #![allow(unused_variables)]
  /// # #[macro_use] extern crate e_num;
  /// # use e_num::ENum;
  /// # #[derive(ENum)]
  /// # enum A {
  /// #   B,
  /// #   C,
  /// #   D,
  /// # }
  /// # let sketchy_number = 0b11;
  /// if let Some(val) = A::try_from_num(sketchy_number) {
  ///   // handle val
  /// } else {
  ///   // handle error
  /// }
  /// ```
  fn try_from_num(num: usize) -> Option<Self>;
  /// Convert self to a serializable number.
  ///
  /// # Examples
  ///
  /// ```
  /// # #[macro_use] extern crate e_num;
  /// # use e_num::ENum;
  /// # #[derive(ENum)]
  /// # enum A {
  /// #   B,
  /// #   C,
  /// # }
  /// let num = A::B.to_num();
  /// // later...
  /// let a = A::from_num(num);
  /// assert!(match a {
  ///   A::B => true,
  ///   _ => false,
  /// });
  /// ```
  fn to_num(&self) -> usize;
}

macro_rules! impl_e_num_num {
  ($($num:ty),*) => {
    $(impl ENum for $num {
      fn try_from_num(num:usize) -> Option<Self> {
        Some(Self::from_num(num))
      }
      fn from_num(num: usize) -> Self {
        num as Self
      }
      fn to_num(&self) -> usize {
        *self as usize
      }
    })*
  };
}

impl_e_num_num!(usize, u64, u32, u16);
