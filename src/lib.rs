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
  #[e_num(start_at = "0")]
  enum B {
    A,
    C(usize),
    D,
  }
  #[test]
  fn basic() {
    assert!(match B::from_num(0) {
      B::A => true,
      _ => false,
    });
  }

  #[test]
  fn value_from_enum() {
    assert!(match B::from_num(0b10101) {
      B::C(v) => v == 0b101,
      _ => false,
    });
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

impl_e_num_num!(usize, u64);
