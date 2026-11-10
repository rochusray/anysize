#![no_std]

use concat_idents::concat_idents;
use core::{
    fmt::{self, Formatter, Write},
    ops::{
        Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Deref,
        DerefMut, Div, DivAssign, Mul, MulAssign, Not, Rem, RemAssign, Shl, ShlAssign, Shr,
        ShrAssign, Sub, SubAssign,
    },
};
#[cfg(feature = "zerocopy")]
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

const UNITS: &[u8] = &[b'K', b'M', b'G', b'T', b'P', b'E', b'Z', b'Y', b'R', b'Q'];

macro_rules! get_bits_from_type_value {
    (u8, U8, $val:expr) => {
        #[cfg(any(all(feature = "u16", feature = "bits"), not(any(feature = "u16", feature = "u32", feature = "usize"))))]
        get_bits_from_type_value!(u16, U16, $val);
        #[cfg(all(not(feature = "u16"), feature = "u32"))]
        get_bits_from_type_value!(u32, U32, $val);
        #[cfg(all(not(feature = "u16"), not(feature = "u32"), feature = "usize"))]
        get_bits_from_type_value!(usize, USize, $val);
    };
    ($ty:ident, $name:ident, $val:expr $(, $feature:literal)?) => {
        concat_idents!(bits_ident = Bits, $name {
            /// Amount of bits, describing the size of this type
            /// Note: The type is at least 16bit
            #[cfg(feature = "bits")]
            pub const BITS: bits_ident = bits_ident::new($val as $ty);

            /// Amount of bits, describing the size of this type
            /// Note: The type is at least 16bit
            #[cfg(not(feature = "bits"))]
            pub const BITS: $ty = $val as $ty;
        });
    }
}

macro_rules! wrap_inner {
    (fn $name:ident $(as $as_ty:ident)? $(-> as $var_ty:ident)?) => {
        #[doc = concat!("Const wrapper for `", stringify!($name), "` function.")]
        #[must_use]
        #[inline(always)]
        pub const fn $name(self, rhs: Self) -> Self {
            Self::new(self.get().$name(rhs.get() $(as $as_ty)?) $(as $var_ty)?)
        }
    };
    (fn $name:ident $(as $as_ty:ident)? -> bits($base_ty:ident as $base_ident:ident)) => {
        concat_idents!(BitsTy = Bits, $base_ident {
            #[doc = concat!("Const wrapper for `", stringify!($name), "` function.")]
            #[must_use]
            #[inline(always)]
            #[cfg(feature = "bits")]
            pub const fn $name(self, rhs: Self) -> BitsTy {
                BitsTy::new(self.get().$name(rhs.get() $(as $as_ty)?) as $base_ty)
            }
        });
        #[doc = concat!("Const wrapper for `", stringify!($name), "` function.")]
        #[must_use]
        #[inline(always)]
        #[cfg(not(feature = "bits"))]
        pub const fn $name(self, rhs: Self) -> $base_ty {
            self.get().$name(rhs.get() $(as $as_ty)?)
        }
    };
    (unsafe fn $name:ident $(as $as_ty:ident)? $(-> as $var_ty:ident)?) => {
        #[doc = concat!("Const wrapper for `", stringify!($name), "` function.")]
        #[must_use]
        #[inline(always)]
        pub const unsafe fn $name(self, rhs: Self) -> Self {
            Self::new(unsafe { self.get().$name(rhs.get() $(as $as_ty)?) } $(as $var_ty)?)
        }
    };
    (fn $name:ident $(as $as_ty:ident)? -> ? $(-> as $var_ty:ident)?) => {
        #[doc = concat!("Const wrapper for `", stringify!($name), "` function.")]
        #[must_use]
        #[inline(always)]
        pub const fn $name(self, rhs: Self) -> Option<Self> {
            match self.get().$name(rhs.get() $(as $as_ty)?) {
                Some(val) => Some(Self::new(val $(as $var_ty)?)),
                None => None
            }
        }
    };
    (fn $name:ident $(as $as_ty:ident)? -> ($ret_ty:ty) $(-> as $var_ty:ident)?) => {
        #[doc = concat!("Const wrapper for `", stringify!($name), "` function.")]
        #[must_use]
        #[inline(always)]
        pub const fn $name(self, rhs: Self) -> (Self, $ret_ty) {
            let tmp = self.get().$name(rhs.get() $(as $as_ty)?);
            (Self::new(tmp.0 $(as $var_ty)?), tmp.1)
        }
    };
    (fn $name:ident(_) $(-> as $var_ty:ident)?) => {
        #[doc = concat!("Const wrapper for `", stringify!($name), "` function.")]
        #[must_use]
        #[inline(always)]
        pub const fn $name(self) -> Self {
            Self::new(self.get().$name() $(as $var_ty)?)
        }
    };
    (fn $name:ident(_) -> bits($base_ty:ident as $base_ident:ident)) => {
        concat_idents!(BitsTy = Bits, $base_ident {
            #[doc = concat!("Const wrapper for `", stringify!($name), "` function.")]
            #[must_use]
            #[inline(always)]
            #[cfg(feature = "bits")]
            pub const fn $name(self) -> BitsTy {
                BitsTy::new(self.get().$name() as $base_ty)
            }
        });
        #[doc = concat!("Const wrapper for `", stringify!($name), "` function.")]
        #[must_use]
        #[inline(always)]
        #[cfg(not(feature = "bits"))]
        pub const fn $name(self) -> $base_ty {
            self.$name()
        }
    };

    (fn $name:ident($wrapped:tt)) => {
        #[doc = concat!("Const wrapper for `", stringify!($wrapped), "` operator.")]
        #[must_use]
        #[inline(always)]
        pub const fn $name(self, rhs: Self) -> Self {
            Self::new(self.get() $wrapped rhs.get())
        }
    };
    (fn $name:ident($wrapped:tt) -> $ret_ty:ty) => {
        #[doc = concat!("Const wrapper for `", stringify!($wrapped), "` operator.")]
        #[must_use]
        #[inline(always)]
        pub const fn $name(self, rhs: Self) -> $ret_ty {
            self.get() $wrapped rhs.get()
        }
    };
    (fn $name:ident($wrapped:tt _)) => {
        #[doc = concat!("Const wrapper for `", stringify!($wrapped), "` operator.")]
        #[must_use]
        #[inline(always)]
        pub const fn $name(self) -> Self {
            Self::new($wrapped self.get())
        }
    };
}
/// For generating conversion functions between data types
#[allow(unused_macros)]
macro_rules! generate_type_conversion_fn {
    {
        $a_base_ident:ident ($a_bit_size:literal) => $b_base_ident:ident ($b_bit_size:literal) as $ident:ident {
            $(#[feature($feature:literal)] $ty:ident as $name:ident),+$(,)?
        }
    } => {
        $(
            #[cfg(feature = $feature)]
            #[cfg_attr(not(feature = $feature), allow(rust_analyzer::inactive_code))]
            const _: () = {
                concat_idents!(a = $a_base_ident, $name {
                    type $a_base_ident = a;
                });
                concat_idents!(b = $b_base_ident, $name {
                    type $b_base_ident = b;
                });
                concat_idents!(into_b = as_, $ident {
                    impl $a_base_ident {
                        #[doc = concat!("Converts a ", stringify!($a_base_ident), stringify!($name), " into a ", stringify!($b_base_ident), stringify!($name))]
                        #[doc = "# Example"]
                        #[doc = "```"]
                        #[doc = concat!("use anysize::", stringify!($a_base_ident), stringify!($name), ";")]
                        #[doc = concat!("let a = ", stringify!($a_base_ident), stringify!($name), "::new(1);")]
                        #[doc = concat!("let ", stringify!($ident), " = a.as_", stringify!($ident), "();")]
                        #[doc = concat!("if ", stringify!($a_bit_size), " > ", stringify!($b_bit_size), " {")]
                        #[doc = concat!("    assert_eq!(*", stringify!($ident), ", *a * (", stringify!($a_bit_size), " / ", stringify!($b_bit_size), ") as ", stringify!($ty), ");")]
                        #[doc = "} else {"]
                        #[doc = concat!("    assert_eq!(*", stringify!($ident), ", (*a).div_ceil((", stringify!($b_bit_size), " / ", stringify!($a_bit_size), ").min(1) as ", stringify!($ty), "));")]
                        #[doc = "}"]
                        #[doc = "```"]
                        #[must_use]
                        pub const fn into_b(self) -> $b_base_ident {
                            if $a_bit_size > $b_bit_size {
                                $b_base_ident::new(self.mul(Self::new(($a_bit_size / $b_bit_size) as $ty)).get())
                            } else {
                                $b_base_ident::new(self.div_ceil((if ($b_bit_size / $a_bit_size) as $ty == 0 { Self::ONE } else { Self::new(($b_bit_size / $a_bit_size) as $ty)})).get())
                            }
                        }
                    }
                    impl Into<$b_base_ident> for $a_base_ident {
                        #[inline(always)]
                        fn into(self) -> $b_base_ident {
                            self.into_b()
                        }
                    }
                });
            };
        )+
    };
}
/// For generating conversion functions between inner types
macro_rules! generate_size_conversion_fn {
    {
        $base_ident:ident as $ident:ident {
            $(#[features($feature_a:literal, $feature_b:literal)] $(#[pointer_size($ptr_size:literal)])? $a_ty:ident as $a_name:ident $cmp:tt $b_ty:ident as $b_name:ident),+$(,)?
        }
    } => {
        $(
            #[cfg(all(feature = $feature_a, feature = $feature_b$(, target_pointer_width = $ptr_size)?))]
            #[cfg_attr(not(all(feature = $feature_a, feature = $feature_b$(, target_pointer_width = $ptr_size)?)), allow(rust_analyzer::inactive_code))]
            const _: () = {
            concat_idents!(a = $base_ident, $a_name {
                type $a_name = a;
            });
            concat_idents!(b = $base_ident, $b_name {
                type $b_name = b;
            });
            concat_idents!(into_b = as_, $b_ty {
                impl $a_name {
                    #[doc = concat!("Converts a ", stringify!($base_ident), stringify!($a_name), " into a ", stringify!($base_ident), stringify!($b_name))]
                    #[doc = "# Example:"]
                    #[doc = "```"]
                    #[doc = concat!("use anysize::", stringify!($base_ident), stringify!($a_name), ";")]
                    #[doc = concat!("let a = ", stringify!($base_ident), stringify!($a_name), "::new(10);")]
                    #[doc = concat!("let b = a.as_", stringify!($b_ty), "();")]
                    #[doc = concat!("assert_eq!(a, b.as_", stringify!($a_ty), "());")]
                    #[doc = "```"]
                    #[must_use]
                    #[inline(always)]
                    pub const fn into_b(self) -> $b_name {
                        $b_name::new(self.get() as $b_ty)
                    }
                }
                impl Into<$b_name> for $a_name {
                    #[inline(always)]
                    fn into(self) -> $b_name {
                        self.into_b()
                    }
                }
            });
            concat_idents!(into_a = as_, $a_ty {
                impl $b_name {
                    #[doc = concat!("Converts a ", stringify!($base_ident), stringify!($b_name), " into a ", stringify!($base_ident), stringify!($a_name))]
                    #[doc = "# Example:"]
                    #[doc = "```"]
                    #[doc = concat!("use anysize::", stringify!($base_ident), stringify!($b_name), ";")]
                    #[doc = concat!("let a = ", stringify!($base_ident), stringify!($b_name), "::new(10);")]
                    #[doc = concat!("let b = a.as_", stringify!($a_ty), "();")]
                    #[doc = "```"]
                    #[must_use]
                    #[inline(always)]
                    pub const fn into_a(self) -> $a_name {
                        $a_name::new(self.get() as $a_ty)
                    }
                }
                impl Into<$a_name> for $b_name {
                    #[inline(always)]
                    fn into(self) -> $a_name {
                        self.into_a()
                    }
                }
            });
        };)+
    };
}
/// Generating the type itself + some basic functionality with itself
macro_rules! generate_base {
    {
        $words:ident = $bit_size:literal;
        struct $base_ident:ident ($short_name:literal): $(
            #[feature($feature:literal)] $var_ty:ident as $var_ident:ident),+$(,)?
    } => {
        $(
            concat_idents!(struct_ident = $base_ident, $var_ident {
                #[doc = "Const zero cost abstraction to add differenciate data size types."]
                #[cfg(feature = $feature)]
                #[repr(transparent)]
                #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
                #[cfg_attr(
                    feature = "zerocopy",
                    derive(FromBytes, Immutable, KnownLayout)
                )]
                pub struct struct_ident {
                   pub(self) inner: $var_ty,
                }

                #[cfg(feature = $feature)]
                impl struct_ident {
                    get_bits_from_type_value!($var_ty, $var_ident, $bit_size);

                    pub const ZERO: Self = Self::new(0);
                    pub const ONE: Self = Self::new(1);
                    pub const MIN: Self = Self::new($var_ty::MIN);
                    pub const MAX: Self = Self::new($var_ty::MAX);

                    #[doc = "Will create a new instance of the given type"]
                    #[doc = "# Examples"]
                    #[doc = "```"]
                    #[doc = concat!("use anysize::", stringify!($base_ident), stringify!($var_ident), ";")]
                    #[doc = concat!("let ", stringify!($words), " = ", stringify!($base_ident), stringify!($var_ident), "::new(10);")]
                    #[doc = concat!("assert_eq!(*", stringify!($words), ", 10);")]
                    #[doc = "```"]
                    #[must_use]
                    #[inline(always)]
                    pub const fn new($words: $var_ty) -> Self {
                       Self { inner: $words }
                    }
                    #[doc = "Gets the value as integer"]
                    #[doc = "# Examples"]
                    #[doc = "```"]
                    #[doc = concat!("use anysize::", stringify!($base_ident), stringify!($var_ident), ";")]
                    #[doc = concat!("let a = ", stringify!($base_ident), stringify!($var_ident), "::new(25);")]
                    #[doc = "assert_eq!(a.get(), 25);"]
                    #[doc = "assert_eq!(a.get(), *a);"]
                    #[doc = "```"]
                    #[must_use]
                    #[inline(always)]
                    pub const fn get(self) -> $var_ty {
                        self.inner
                    }
                    #[doc = "Creates new instance, depending on the size of a type"]
                    #[cfg_attr(feature = "bytes", doc = "# Examples")]
                    #[cfg_attr(feature = "bytes", doc = "```")]
                    #[cfg_attr(feature = "bytes", doc = "use anysize::*;")]
                    #[cfg_attr(feature = "bytes", doc = concat!("assert_eq!(*Into::<Bytes", stringify!($var_ident), ">::into(", stringify!($base_ident), stringify!($var_ident), "::of::<u8>()) as usize, size_of::<u8>().next_multiple_of(", stringify!($bit_size), "usize.div_ceil(8)));"))]
                    #[cfg_attr(feature = "bytes", doc = concat!("assert_eq!(*Into::<Bytes", stringify!($var_ident), ">::into(", stringify!($base_ident), stringify!($var_ident), "::of::<u32>()) as usize, size_of::<u32>().next_multiple_of(", stringify!($bit_size), "usize.div_ceil(8)));"))]
                    #[cfg_attr(feature = "bytes", doc = concat!("assert_eq!(*Into::<Bytes", stringify!($var_ident), ">::into(", stringify!($base_ident), stringify!($var_ident), "::of::<usize>()) as usize, size_of::<usize>().next_multiple_of(", stringify!($bit_size), "usize.div_ceil(8)));"))]
                    #[cfg_attr(feature = "bytes", doc = "```")]
                    #[must_use]
                    #[inline]
                    pub const fn of<T>() -> Self {
                        let size = if $bit_size < 8 {
                            size_of::<T>() * ((8 / $bit_size) as usize)
                        } else if $bit_size == 8 {
                            size_of::<T>()
                        } else {
                            size_of::<T>().div_ceil(($bit_size / 8) as usize)
                        };
                        assert!(size <= $var_ty::MAX as usize);
                        Self::new(size as $var_ty)
                    }

                    wrap_inner!(fn add(+));
                    wrap_inner!(fn checked_add -> ?);
                    wrap_inner!(fn wrapping_add);
                    wrap_inner!(fn saturating_add);
                    wrap_inner!(fn overflowing_add -> (bool));
                    wrap_inner!(fn strict_add);
                    wrap_inner!(unsafe fn unchecked_add);

                    wrap_inner!(fn sub(-));
                    wrap_inner!(fn checked_sub -> ?);
                    wrap_inner!(fn wrapping_sub);
                    wrap_inner!(fn saturating_sub);
                    wrap_inner!(fn overflowing_sub -> (bool));
                    wrap_inner!(fn strict_sub);
                    wrap_inner!(unsafe fn unchecked_sub);

                    wrap_inner!(fn mul(*));
                    wrap_inner!(fn checked_mul -> ?);
                    wrap_inner!(fn wrapping_mul);
                    wrap_inner!(fn saturating_mul);
                    wrap_inner!(fn overflowing_mul -> (bool));
                    wrap_inner!(fn strict_mul);
                    wrap_inner!(unsafe fn unchecked_mul);

                    wrap_inner!(fn div(/));
                    wrap_inner!(fn checked_div -> ?);
                    wrap_inner!(fn wrapping_div);
                    wrap_inner!(fn saturating_div);
                    wrap_inner!(fn overflowing_div -> (bool));
                    wrap_inner!(fn strict_div);
                    wrap_inner!(fn div_ceil);

                    wrap_inner!(fn rem(%));
                    wrap_inner!(fn checked_rem -> ?);
                    wrap_inner!(fn wrapping_rem);
                    wrap_inner!(fn overflowing_rem -> (bool));
                    wrap_inner!(fn strict_rem);

                    wrap_inner!(fn abs_diff);
                    wrap_inner!(fn ilog -> as $var_ty);
                    wrap_inner!(fn ilog10(_) -> as $var_ty);
                    wrap_inner!(fn ilog2(_) -> as $var_ty);
                    wrap_inner!(fn isqrt(_) -> as $var_ty);
                    wrap_inner!(fn midpoint);
                    wrap_inner!(fn next_multiple_of);
                    wrap_inner!(fn next_power_of_two(_));

                    wrap_inner!(fn eq(==) -> bool);
                    wrap_inner!(fn ne(!=) -> bool);
                    wrap_inner!(fn gt(>) -> bool);
                    wrap_inner!(fn lt(<) -> bool);
                    wrap_inner!(fn ge(>=) -> bool);
                    wrap_inner!(fn le(<=) -> bool);

                    wrap_inner!(fn bitand(&));
                    wrap_inner!(fn bitor(|));
                    wrap_inner!(fn bitxor(^));
                    wrap_inner!(fn not(!_));

                    wrap_inner!(fn shl(<<));
                    wrap_inner!(fn checked_shl as u32 -> ?);
                    wrap_inner!(fn wrapping_shl as u32);
                    wrap_inner!(fn overflowing_shl as u32 -> (bool));
                    wrap_inner!(fn unbounded_shl as u32);
                    wrap_inner!(fn strict_shl as u32);
                    wrap_inner!(unsafe fn unchecked_shl as u32);

                    wrap_inner!(fn shr(>>));
                    wrap_inner!(fn checked_shr as u32 -> ?);
                    wrap_inner!(fn wrapping_shr as u32);
                    wrap_inner!(fn overflowing_shr as u32 -> (bool));
                    wrap_inner!(fn unbounded_shr as u32);
                    wrap_inner!(fn strict_shr as u32);
                    wrap_inner!(unsafe fn unchecked_shr as u32);

                    wrap_inner!(fn bit_width(_) -> bits($var_ty as $var_ident));
                    wrap_inner!(fn count_ones(_) -> bits($var_ty as $var_ident));
                    wrap_inner!(fn count_zeros(_) -> bits($var_ty as $var_ident));
                    wrap_inner!(fn isolate_highest_one(_) -> bits($var_ty as $var_ident));
                    wrap_inner!(fn isolate_lowest_one(_) -> bits($var_ty as $var_ident));
                    wrap_inner!(fn leading_ones(_) -> bits($var_ty as $var_ident));
                    wrap_inner!(fn leading_zeros(_) -> bits($var_ty as $var_ident));
                    wrap_inner!(fn reverse_bits(_));
                    wrap_inner!(fn swap_bytes(_));
                    wrap_inner!(fn to_be(_));
                    wrap_inner!(fn to_le(_));
                    wrap_inner!(fn rotate_left as u32);
                    wrap_inner!(fn rotate_right as u32);

                    /// Const min
                    #[must_use]
                    #[inline(always)]
                    pub const fn min(self, other: Self) -> Self {
                        if self.lt(other) {
                            self
                        } else {
                            other
                        }
                    }
                    /// Const max
                    #[must_use]
                    #[inline(always)]
                    pub const fn max(self, other: Self) -> Self {
                        if self.gt(other) {
                            self
                        } else {
                            other
                        }
                    }
                    /// Const clamp
                    #[must_use]
                    #[inline(always)]
                    pub const fn clamp(self, min: Self, max: Self) -> Self {
                        assert!(min.le(max));
                        if self.le(min) {
                            min
                        } else if self.ge(max) {
                            max
                        } else {
                            self
                        }
                    }

                    /// Implements zerocopy::IntoBytes
                    #[must_use]
                    #[inline(always)]
                    #[cfg(feature = "zerocopy")]
                    pub fn as_byte_slice(&self) -> &[u8] {
                        self.inner.as_bytes()
                    }
                }

                #[cfg(feature = $feature)]
                impl Add for struct_ident {
                    type Output = Self;

                    #[inline(always)]
                    fn add(self, rhs: Self) -> Self::Output {
                        self.add(rhs)
                    }
                }
                #[cfg(feature = $feature)]
                impl AddAssign for struct_ident {
                    #[inline(always)]
                    fn add_assign(&mut self, rhs: Self) {
                        *self = self.add(rhs);
                    }
                }
                #[cfg(feature = $feature)]
                impl Sub for struct_ident {
                    type Output = Self;

                    #[inline(always)]
                    fn sub(self, rhs: Self) -> Self::Output {
                        self.sub(rhs)
                    }
                }
                #[cfg(feature = $feature)]
                impl SubAssign for struct_ident {
                    #[inline(always)]
                    fn sub_assign(&mut self, rhs: Self) {
                        *self = self.sub(rhs);
                    }
                }
                #[cfg(feature = $feature)]
                impl Mul for struct_ident {
                    type Output = Self;

                    #[inline(always)]
                    fn mul(self, rhs: Self) -> Self::Output {
                        self.mul(rhs)
                    }
                }
                #[cfg(feature = $feature)]
                impl MulAssign for struct_ident {
                    #[inline(always)]
                    fn mul_assign(&mut self, rhs: Self) {
                        *self = self.mul(rhs);
                    }
                }
                #[cfg(feature = $feature)]
                impl Div for struct_ident {
                    type Output = Self;

                    #[inline(always)]
                    fn div(self, rhs: Self) -> Self::Output {
                        self.div(rhs)
                    }
                }
                #[cfg(feature = $feature)]
                impl DivAssign for struct_ident {
                    #[inline(always)]
                    fn div_assign(&mut self, rhs: Self) {
                        *self = self.div(rhs);
                    }
                }
                #[cfg(feature = $feature)]
                impl Rem for struct_ident {
                    type Output = Self;

                    #[inline(always)]
                    fn rem(self, rhs: Self) -> Self::Output {
                        self.rem(rhs)
                    }
                }
                #[cfg(feature = $feature)]
                impl RemAssign for struct_ident {
                    #[inline(always)]
                    fn rem_assign(&mut self, rhs: Self) {
                        *self = self.rem(rhs);
                    }
                }

                #[cfg(feature = $feature)]
                impl BitAnd for struct_ident {
                    type Output = Self;

                    #[inline(always)]
                    fn bitand(self, rhs: Self) -> Self::Output {
                        self.bitand(rhs)
                    }
                }
                #[cfg(feature = $feature)]
                impl BitAndAssign for struct_ident {
                    #[inline(always)]
                    fn bitand_assign(&mut self, rhs: Self) {
                        *self = self.bitand(rhs);
                    }
                }
                #[cfg(feature = $feature)]
                impl BitOr for struct_ident {
                    type Output = Self;

                    #[inline(always)]
                    fn bitor(self, rhs: Self) -> Self::Output {
                        self.bitor(rhs)
                    }
                }
                #[cfg(feature = $feature)]
                impl BitOrAssign for struct_ident {
                    #[inline(always)]
                    fn bitor_assign(&mut self, rhs: Self) {
                        *self = self.bitor(rhs);
                    }
                }
                #[cfg(feature = $feature)]
                impl BitXor for struct_ident {
                    type Output = Self;

                    #[inline(always)]
                    fn bitxor(self, rhs: Self) -> Self::Output {
                        self.bitxor(rhs)
                    }
                }
                #[cfg(feature = $feature)]
                impl BitXorAssign for struct_ident {
                    #[inline(always)]
                    fn bitxor_assign(&mut self, rhs: Self) {
                        *self = self.bitxor(rhs);
                    }
                }
                #[cfg(feature = $feature)]
                impl Not for struct_ident {
                    type Output = Self;

                    #[inline(always)]
                    fn not(self) -> Self::Output {
                        self.not()
                    }
                }

                #[cfg(feature = $feature)]
                impl Shl for struct_ident {
                    type Output = Self;

                    #[inline(always)]
                    fn shl(self, rhs: Self) -> Self::Output {
                        self.shl(rhs)
                    }
                }
                #[cfg(feature = $feature)]
                impl ShlAssign for struct_ident {
                    #[inline(always)]
                    fn shl_assign(&mut self, rhs: Self) {
                        *self = self.shl(rhs);
                    }
                }
                #[cfg(feature = $feature)]
                impl Shr for struct_ident {
                    type Output = Self;

                    #[inline(always)]
                    fn shr(self, rhs: Self) -> Self::Output {
                        self.shr(rhs)
                    }
                }
                #[cfg(feature = $feature)]
                impl ShrAssign for struct_ident {
                    #[inline(always)]
                    fn shr_assign(&mut self, rhs: Self) {
                        *self = self.shr(rhs);
                    }
                }

                #[cfg(feature = $feature)]
                impl Deref for struct_ident {
                    type Target = $var_ty;

                    fn deref(&self) -> &Self::Target {
                        &self.inner
                    }
                }
                #[cfg(feature = $feature)]
                impl DerefMut for struct_ident {
                    fn deref_mut(&mut self) -> &mut Self::Target {
                        &mut self.inner
                    }
                }

                #[cfg(feature = $feature)]
                impl Into<$var_ty> for struct_ident {
                    #[inline(always)]
                    fn into(self) -> $var_ty {
                        self.get()
                    }
                }
                #[cfg(feature = $feature)]
                impl From<$var_ty> for struct_ident {
                    #[inline(always)]
                    fn from(value:$var_ty) -> struct_ident {
                        Self::new(value)
                    }
                }

                #[cfg(feature = $feature)]
                impl fmt::Binary for struct_ident {
                    #[inline(always)]
                    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                        fmt::Binary::fmt(&self.inner, f)
                    }
                }
                #[cfg(feature = $feature)]
                impl fmt::UpperHex for struct_ident {
                    #[inline(always)]
                    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                        fmt::Binary::fmt(&self.inner, f)
                    }
                }
                #[cfg(feature = $feature)]
                impl fmt::LowerHex for struct_ident {
                    #[inline(always)]
                    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                        fmt::LowerHex::fmt(&self.inner, f)
                    }
                }
                #[cfg(feature = $feature)]
                impl fmt::Octal for struct_ident {
                    #[inline(always)]
                    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                        fmt::Octal::fmt(&self.inner, f)
                    }
                }
                #[cfg(feature = $feature)]
                impl fmt::UpperExp for struct_ident {
                    #[inline(always)]
                    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                        fmt::UpperExp::fmt(&self.inner, f)
                    }
                }
                #[cfg(feature = $feature)]
                impl fmt::LowerExp for struct_ident {
                    #[inline(always)]
                    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                        fmt::LowerExp::fmt(&self.inner, f)
                    }
                }
                #[cfg(feature = $feature)]
                impl fmt::Display for struct_ident {
                    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                        let mut integer = self.get();
                        let mut fraction = 0;
                        let mut unit: u8 = u8::MAX;
                        const ONE_K: $var_ty = 1024u16 as $var_ty;
                        if ONE_K > 0 {
                            while integer >= 1024u16 as $var_ty {
                                fraction *= 1024u16 as $var_ty;
                                fraction += integer % 1024u16 as $var_ty;
                                integer /= 1024u16 as $var_ty;
                                unit = unit.wrapping_add(1);
                            }
                        }

                        if f.sign_minus() {
                            write!(f, "-")?;
                        } else if f.sign_plus() {
                           write!(f, "+")?;
                        }
                        write!(f, "{integer}")?;

                        if unit != 255 {
                            let max_possible_decimals = (unit as usize + 1) * 4;
                            let required_decimals = f.precision().unwrap_or(2);
                            let decimals = max_possible_decimals.min(required_decimals);
                            if let Some(decimals_to_shave) = max_possible_decimals.checked_sub(required_decimals) {
                                for _ in 0..decimals_to_shave {
                                    fraction /= 10;
                                    if fraction == 0 {
                                        break;
                                    }
                                }
                            }

                            if fraction > 0 {
                                write!(f, ".{fraction:0decimals$}")?;
                            }
                        }

                        f.write_char(' ')?;
                        if unit < UNITS.len() as u8 {
                            f.write_char(UNITS[unit as usize] as char)?;
                            f.write_char('i')?;
                        }
                        f.write_str($short_name)
                    }
                }
                #[cfg(feature = $feature)]
                impl fmt::Debug for struct_ident {
                    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                        fmt::Display::fmt(&self.inner, f)?;
                        f.write_char(' ')?;
                        f.write_str($short_name)
                    }
                }
            });
        )+
    };
}
macro_rules! generate {
    ($words:ident = $bit_size:literal as $name:ident ($short_name:literal) $(, impl Into<$(#[feature($feature:literal)] $impl_ty:ident ($impl_bits:literal) as $impl_words:ident),+$(,)?>)*) => {
        generate_base! {
            $words = $bit_size;
            struct $name ($short_name):
                #[feature("u8")] u8 as U8,
                #[feature("u16")] u16 as U16,
                #[feature("u32")] u32 as U32,
                #[feature("u64")] u64 as U64,
                #[feature("u128")] u128 as U128,
                #[feature("usize")] usize as USize,
        }
        generate_size_conversion_fn! {
            $name as $words {
                #[features("u8", "usize")] u8 as U8 < usize as USize,
                #[features("u8", "u16")] u8 as U8 < u16 as U16,
                #[features("u8", "u32")] u8 as U8 < u32 as U32,
                #[features("u8", "u64")] u8 as U8 < u64 as U64,
                #[features("u8", "u128")] u8 as U8 < u128 as U128,
                #[features("u16", "u32")] u16 as U16 < u32 as U32,
                #[features("u16", "u64")] u16 as U16 < u64 as U64,
                #[features("u16", "u128")] u16 as U16 < u128 as U128,
                #[features("u32", "u64")] u32 as U32 < u64 as U64,
                #[features("u32", "u128")] u32 as U32 < u128 as U128,
                #[features("u64", "u128")] u64 as U64 < u128 as U128,

                #[features("usize", "u16")] #[pointer_size("16")] usize as USize == u16 as U16,
                #[features("usize", "u32")] #[pointer_size("16")] usize as USize < u32 as U32,
                #[features("usize", "u64")] #[pointer_size("16")] usize as USize < u64 as U64,

                #[features("usize", "u16")] #[pointer_size("32")] usize as USize > u16 as U16,
                #[features("usize", "u32")] #[pointer_size("32")] usize as USize == u32 as U32,
                #[features("usize", "u64")] #[pointer_size("32")] usize as USize < u64 as U64,

                #[features("usize", "u16")] #[pointer_size("64")] usize as USize > u16 as U16,
                #[features("usize", "u32")] #[pointer_size("64")] usize as USize > u32 as U32,
                #[features("usize", "u64")] #[pointer_size("64")] usize as USize == u64 as U64,

                #[features("usize", "u128")] usize as USize < u128 as U128,
            }
        }
        $(
            $(
                #[cfg(feature = $feature)]
                #[cfg_attr(not(feature = $feature), allow(rust_analyzer::inactive_code))]
                generate_type_conversion_fn! {
                    $name ($bit_size) => $impl_ty ($impl_bits) as $impl_words {
                        #[feature("u8")] u8 as U8,
                        #[feature("u16")] u16 as U16,
                        #[feature("u32")] u32 as U32,
                        #[feature("u64")] u64 as U64,
                        #[feature("u128")] u128 as U128,
                        #[feature("usize")] usize as USize,
                    }
                }
            )+
        )?
    };
}

#[cfg(feature = "bits")]
generate!(bits = 1 as Bits ("b"), impl Into<
    #[feature("bytes")] Bytes (8) as bytes,
    #[feature("hwords")] HWords (16) as hwords,
    #[feature("words")] Words (32) as words,
    #[feature("dwords")] DWords (64) as dwords,
    #[feature("qwords")] QWords (128) as qwords,
    #[feature("ywords")] YWords (256) as ywords,
    #[feature("zwords")] ZWords (512) as zwords,
>);

#[cfg(feature = "bytes")]
generate!(bytes = 8 as Bytes ("B"), impl Into<
    #[feature("bits")] Bits (1) as bits,
    #[feature("hwords")] HWords (16) as hwords,
    #[feature("words")] Words (32) as words,
    #[feature("dwords")] DWords (64) as dwords,
    #[feature("qwords")] QWords (128) as qwords,
    #[feature("ywords")] YWords (256) as ywords,
    #[feature("zwords")] ZWords (512) as zwords,
>);

#[cfg(feature = "hwords")]
generate!(hwords = 16 as HWords ("HW"), impl Into<
    #[feature("bits")] Bits (1) as bits,
    #[feature("bytes")] Bytes (8) as bytes,
    #[feature("words")] Words (32) as words,
    #[feature("dwords")] DWords (64) as dwords,
    #[feature("qwords")] QWords (128) as qwords,
    #[feature("ywords")] YWords (256) as ywords,
    #[feature("zwords")] ZWords (512) as zwords,
>);

#[cfg(feature = "words")]
generate!(words = 32 as Words ("W"), impl Into<
    #[feature("bits")] Bits (1) as bits,
    #[feature("bytes")] Bytes (8) as bytes,
    #[feature("hwords")] HWords (16) as hwords,
    #[feature("dwords")] DWords (64) as dwords,
    #[feature("qwords")] QWords (128) as qwords,
    #[feature("ywords")] YWords (256) as ywords,
    #[feature("zwords")] ZWords (512) as zwords,
>);

#[cfg(feature = "dwords")]
generate!(dwords = 64 as DWords ("DW"), impl Into<
    #[feature("bits")] Bits (1) as bits,
    #[feature("bytes")] Bytes (8) as bytes,
    #[feature("hwords")] HWords (16) as hwords,
    #[feature("words")] Words (32) as words,
    #[feature("qwords")] QWords (128) as qwords,
    #[feature("ywords")] YWords (256) as ywords,
    #[feature("zwords")] ZWords (512) as zwords,
>);

#[cfg(feature = "qwords")]
generate!(qwords = 128 as QWords ("QW"), impl Into<
    #[feature("bits")] Bits (1) as bits,
    #[feature("bytes")] Bytes (8) as bytes,
    #[feature("hwords")] HWords (16) as hwords,
    #[feature("words")] Words (32) as words,
    #[feature("dwords")] DWords (64) as dwords,
    #[feature("ywords")] YWords (256) as ywords,
    #[feature("zwords")] ZWords (512) as zwords,
>);

#[cfg(feature = "ywords")]
generate!(ywords = 256 as YWords ("YW"), impl Into<
    #[feature("bits")] Bits (1) as bits,
    #[feature("bytes")] Bytes (8) as bytes,
    #[feature("hwords")] HWords (16) as hwords,
    #[feature("words")] Words (32) as words,
    #[feature("dwords")] DWords (64) as dwords,
    #[feature("qwords")] QWords (128) as qwords,
    #[feature("zwords")] ZWords (512) as zwords,
>);

#[cfg(feature = "zwords")]
generate!(zwords = 512 as ZWords ("ZW"), impl Into<
    #[feature("bits")] Bits (1) as bits,
    #[feature("bytes")] Bytes (8) as bytes,
    #[feature("hwords")] HWords (16) as hwords,
    #[feature("words")] Words (32) as words,
    #[feature("dwords")] DWords (64) as dwords,
    #[feature("qwords")] QWords (128) as qwords,
    #[feature("ywords")] YWords (256) as ywords,
>);

#[test]
fn test() {
    #![allow(unused)]
    let a: u128 = 10;
    let b: u32 = 20;

    a.bit_width();
    a.count_ones();
    a.count_zeros();
    a.isolate_highest_one();
    a.isolate_lowest_one();
    a.leading_ones();
    a.leading_zeros();
    a.reverse_bits();
    a.rotate_left(b);
    a.rotate_right(b);
    a.swap_bytes();
    a.to_be();
    a.to_le();
}
