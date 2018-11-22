extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate darling;

use proc_macro::TokenStream;

#[proc_macro_derive(ENum, attributes(e_num))]
pub fn e_num_derive(input: TokenStream) -> TokenStream {
  let ast = syn::parse(input).unwrap();

  impl_e_num(&ast)
}

#[derive(Debug)]
enum VariantStyle {
  /// A single tuple variant with type T where T: ENum
  /// e.g.
  /// ```ignore
  /// A(usize),
  /// ```
  Field(syn::Type),
  /// A variant with nothing special about it
  /// e.g.
  /// ```ignore
  /// A,
  /// ```
  Unit,
  /// A variant with a discriminant
  /// e.g
  /// ```ignore
  /// A = 1,
  /// ```
  Constant(syn::Expr),
}
#[derive(Debug)]
struct Variant {
  style: VariantStyle,
  name: syn::Ident,
  constant_name: syn::Ident,
}

impl Variant {
  pub fn from_var(var: Var) -> Variant {
    let style = if let Some(AttrExpr(disc)) = var.constant {
      VariantStyle::Constant(disc)
    } else {
      let fields = var.fields;
      use darling::ast::Style;
      match fields.style {
        Style::Tuple => {
          if fields.fields.len() != 1 {
            panic!("Invalid fields for");
          }
          VariantStyle::Field(fields.fields.first().unwrap().clone())
        }
        Style::Unit => VariantStyle::Unit,
        Style::Struct => panic!("ENum can't have a struct variant"),
      }
    };
    let const_ident = syn::Ident::new(
      &format!("{}_MASK", var.ident.to_string().to_uppercase()),
      syn::export::Span::call_site(),
    );
    Variant {
      name: var.ident,
      constant_name: const_ident,
      style,
    }
  }
}

fn round_up(num_to_round: usize) -> usize {
  let mut v = num_to_round;
  v -= 1;
  v |= v >> 1;
  v |= v >> 2;
  v |= v >> 4;
  v |= v >> 8;
  v |= v >> 16;
  v |= v >> 32;
  v += 1;
  v
}

fn impl_e_num(ast: &syn::DeriveInput) -> TokenStream {
  let name = &ast.ident;
  let Attr { start_at, data } = Attr::from_derive_input(ast)
    .unwrap_or_else(|err| panic!("Error while parsing attributes: {}", err));
  let start_at = start_at.0;
  let data = data
    .take_enum()
    .unwrap_or_else(|| panic!("Can't derive struct for ENum"));
  let leading = (round_up(data.len() + start_at) - 1).leading_zeros() as usize;
  let mask_size = 64 - leading;
  let vars = {
    let mut vec = data.into_iter().map(Variant::from_var).collect::<Vec<_>>();
    vec.sort_by(|var1, var2| {
      use std::cmp::Ordering;
      match (&var1.style, &var2.style) {
        (VariantStyle::Constant(_), VariantStyle::Constant(_)) => Ordering::Equal,
        (VariantStyle::Constant(_), _) => Ordering::Greater,
        (_, VariantStyle::Constant(_)) => Ordering::Less,
        _ => Ordering::Equal,
      }
    });
    vec
  };
  let const_names = vars.iter().map(|var| &var.constant_name);
  let const_vals = vars
    .iter()
    .enumerate()
    .map(|(i, Variant { style, .. })| match style {
      VariantStyle::Constant(expr) => quote! { #expr },
      VariantStyle::Unit | VariantStyle::Field(_) => {
        let num = start_at + i;
        quote! { #num }
      }
    }).collect::<Vec<_>>();
  let const_defs = quote! {
    #(const #const_names: usize = #const_vals;)*
  };
  let checks = vars.iter().map(
    |Variant {
       constant_name,
       style,
       ..
     }| match style {
      VariantStyle::Constant(_) => quote! { num == #constant_name },
      VariantStyle::Field(_) | VariantStyle::Unit => {
        quote! { num << #leading >> #leading == #constant_name }
      }
    },
  );
  let outputs = vars.iter().map(
    |Variant {
       name: var_name,
       style,
       ..
     }| match style {
      VariantStyle::Constant(_) | VariantStyle::Unit => quote! { Some(#name::#var_name) },
      VariantStyle::Field(ty) => {
        quote! { <#ty as ENum>::try_from_num(num >> #mask_size).map(|val| #name::#var_name(val)) }
      }
    },
  );
  let matches = vars.iter().map(
    |Variant {
       name: var_name,
       style,
       ..
     }| match style {
      VariantStyle::Field(_) => quote! { #name::#var_name(val) },
      VariantStyle::Constant(_) | VariantStyle::Unit => quote! { #name::#var_name },
    },
  );
  let converts = vars.iter().map(
    |Variant {
       constant_name,
       style,
       ..
     }| match style {
      VariantStyle::Field(ty) => {
        quote! { <#ty as ENum>::to_num(val) << #mask_size | #constant_name }
      }
      VariantStyle::Constant(_) | VariantStyle::Unit => quote! { #constant_name },
    },
  );
  let gen = quote! {
    impl ENum for #name {
      fn try_from_num(num: usize) -> Option<Self> {
        #const_defs
        #(if #checks {
          #outputs
        } else)* {
          None
        }
      }
      fn from_num(num: usize) -> Self {
        if let Some(val) = Self::try_from_num(num) {
          val
        } else {
          panic!(concat!("Failure to parse number into ", stringify!(#name)));
        }
      }
      fn to_num(&self) -> usize {
        #const_defs
        match self {
          #(#matches => {
            #converts
          }),*
        }
      }
    }

    impl From<usize> for #name {
      fn from(num: usize) -> Self {
        Self::from_num(num)
      }
    }
  };
  gen.into()
}

use darling::{FromDeriveInput, FromMeta, FromVariant};

#[derive(Default, Debug)]
struct AttrNum(usize);

impl FromMeta for AttrNum {
  fn from_value(value: &syn::Lit) -> darling::Result<Self> {
    match value {
      syn::Lit::Int(int) => Ok(AttrNum(int.value() as usize)),
      _ => panic!("Attribute value must be an integer literal"),
    }
  }
}

#[derive(Debug)]
struct AttrExpr(syn::Expr);

impl FromMeta for AttrExpr {
  fn from_value(value: &syn::Lit) -> darling::Result<Self> {
    Ok(AttrExpr(syn::Expr::Lit(syn::ExprLit {
      attrs: vec![],
      lit: value.clone(),
    })))
  }
}

#[derive(FromVariant, Debug)]
#[darling(attributes(e_num), forward_attrs(allow, doc, cfg))]
struct Var {
  #[darling(default)]
  constant: Option<AttrExpr>,
  ident: syn::Ident,
  fields: darling::ast::Fields<syn::Type>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(e_num), forward_attrs(allow, doc, cfg))]
struct Attr {
  #[darling(default)]
  pub start_at: AttrNum,
  pub data: darling::ast::Data<Var, ()>,
}
