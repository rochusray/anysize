#![no_std]

use core::{
    fmt::{self, Formatter, Write},
    ops::{Add, AddAssign, Deref, DerefMut, Sub, SubAssign},
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
        concat_idents::concat_idents!(bits_ident = Bits, $name {
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
                concat_idents::concat_idents!(a = $a_base_ident, $name {
                    type $a_base_ident = a;
                });
                concat_idents::concat_idents!(b = $b_base_ident, $name {
                    type $b_base_ident = b;
                });
                concat_idents::concat_idents!(into_b = as_, $ident {
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
                        #[doc = concat!("    assert_eq!(*", stringify!($ident), ", *a.div_ceil((", stringify!($b_bit_size), " / ", stringify!($a_bit_size), ").min(1) as ", stringify!($ty), "));")]
                        #[doc = "}"]
                        #[doc = "```"]
                        #[must_use]
                        pub const fn into_b(self) -> $b_base_ident {
                            if $a_bit_size > $b_bit_size {
                                $b_base_ident::new(self.mul(($a_bit_size / $b_bit_size) as $ty).get())
                            } else {
                                $b_base_ident::new(self.div_ceil((if ($b_bit_size / $a_bit_size) as $ty == 0 { 1 } else { $b_bit_size / $a_bit_size }) as $ty).get())
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
            concat_idents::concat_idents!(a = $base_ident, $a_name {
                type $a_name = a;
            });
            concat_idents::concat_idents!(b = $base_ident, $b_name {
                type $b_name = b;
            });
            concat_idents::concat_idents!(into_b = as_, $b_ty {
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
            concat_idents::concat_idents!(into_a = as_, $a_ty {
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
            concat_idents::concat_idents!(struct_ident = $base_ident, $var_ident {
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
                    #[doc = "Returns the next power of two"]
                    #[must_use]
                    #[inline(always)]
                    pub const fn next_power_of_two(self) -> Self {
                        Self::new(self.inner.next_power_of_two())
                    }

                    /// Const addition
                    #[must_use]
                    #[inline(always)]
                    pub const fn add(self, other: Self) -> Self {
                        Self::new(self.inner + other.inner)
                    }
                    /// Const subtraction
                    #[must_use]
                    #[inline(always)]
                    pub const fn sub(self, other: Self) -> Self {
                        Self::new(self.inner - other.inner)
                    }
                    /// Const multiplication with integer
                    #[must_use]
                    #[inline(always)]
                    pub const fn mul(self, other: $var_ty) -> Self {
                        Self::new(self.inner * other)
                    }
                    /// Const division with integer
                    #[must_use]
                    #[inline(always)]
                    pub const fn div(self, other: $var_ty) -> Self {
                        Self::new(self.inner / other)
                    }
                    /// Const ceil rounding devision with integer
                    #[must_use]
                    #[inline(always)]
                    pub const fn div_ceil(self, other: $var_ty) -> Self {
                        Self::new(self.inner.div_ceil(other))
                    }

                    /// Const eq
                    #[must_use]
                    #[inline(always)]
                    pub const fn eq(self, other: Self) -> bool {
                        self.inner == other.inner
                    }
                    /// Const ne
                    #[must_use]
                    #[inline(always)]
                    pub const fn ne(self, other: Self) -> bool {
                        self.inner != other.inner
                    }
                    /// Const gt
                    #[must_use]
                    #[inline(always)]
                    pub const fn gt(self, other: Self) -> bool {
                        self.inner > other.inner
                    }
                    /// Const ge
                    #[must_use]
                    #[inline(always)]
                    pub const fn ge(self, other: Self) -> bool {
                        self.inner >= other.inner
                    }
                    /// Const lt
                    #[must_use]
                    #[inline(always)]
                    pub const fn lt(self, other: Self) -> bool {
                        self.inner < other.inner
                    }
                    /// Const le
                    #[must_use]
                    #[inline(always)]
                    pub const fn le(self, other: Self) -> bool {
                        self.inner <= other.inner
                    }

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
