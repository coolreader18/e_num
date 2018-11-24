# e_num

> Serialize enums into numbers.

## Warning

This library works with variant fields (e.g. `Variant1(u64)`) by bitshifting the
number representation of the contained value over enough so that the tagging can
fit on the right of the number. If you're dealing with very large numbers in the
fields or have a ton of variants, data on the left side of the value will likely
be lost.

## Usage

```rust
#[macro_use]
extern crate e_num;

use e_num::ENum;

#[derive(ENum)]
enum A {
  B,
  C(u64),
}

fn main() {
  let b: usize = A::B.to_num();
  println!("b as a number: {:#b}", b);
  let b = A::from_num(b);
  assert!(match b {
    A::B => true,
    _ => false,
  });
  let c = A::C(85).to_num();
  println!("c as a number: {:#b}", c);
  let c = A::from_num(c);
  assert!(match c {
    A::C(inner) => {
      assert_eq!(inner, 85);
      true
    }
    _ => false,
  });
}
```

## License

This project is licensed under the MIT license. See the [LICENSE](LICENSE) file
for more details.
