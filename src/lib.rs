#![no_std]

use core::{
    fmt::{self, Formatter, Write},
    ops::{Add, AddAssign, Sub, SubAssign},
};

const UNITS: &[u8] = &[b'K', b'M', b'G', b'T', b'P'];

/// For generating conversion functions between data types
macro_rules! generate_type_conversion_fn {
    {
        $vis:vis $a_base_ident:ident => $b_base_ident:ident as $ident:ident {
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
                        /// Converts one kind of data size type into another
                        /// # Example
                        /// ```
                        /// use anysize::*;
                        /// let words = WordsU32::new(10);
                        /// let bytes = words.as_bytes();
                        /// assert_eq!(bytes.get(), 40);
                        /// let bits = bytes.as_bits();
                        /// assert_eq!(bits.get(), 320);
                        /// assert_eq!(words.as_bits().get(), bits.get());
                        /// ```
                        #[must_use]
                        $vis const fn into_b(self) -> $b_base_ident {
                            if $a_base_ident::BITS.gt($b_base_ident::BITS) {
                                $b_base_ident::new(self.mul($a_base_ident::BITS.get() / $b_base_ident::BITS.get()).get())
                            } else {
                                $b_base_ident::new(self.div_ceil($b_base_ident::BITS.get() / $a_base_ident::BITS.get()).get())
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
        $vis:vis $base_ident:ident as $ident:ident {
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
                    /// Converts the size of an object
                    /// # Panics:
                    /// Will panic if value won't fit in new type
                    /// # Example:
                    /// ```
                    /// use anysize::*;
                    /// let bits = BitsU8::new(10);
                    /// let bits = bits.as_u32(); // Can't panic, because size_of::<u8>() < size_of::<u32>()
                    /// let bits = bits + BitsU32::new(100000);
                    /// // bits.as_u8(); Will panic here
                    /// ```
                    #[must_use]
                    #[inline(always)]
                    $vis const fn into_b(self) -> $b_name {
                        if 1 $cmp 0 {
                            assert!(self.get() <= $b_ty::MAX as $a_ty);
                        }
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
                    /// Converts the size of an object
                    /// # Panics:
                    /// Will panic if value won't fit in new type
                    /// # Example:
                    /// ```
                    /// use anysize::*;
                    /// let bits = BitsU8::new(10);
                    /// let bits = bits.as_u32(); // Can't panic, because size_of::<u8>() < size_of::<u32>()
                    /// let bits = bits + BitsU32::new(100000);
                    /// // bits.as_u8(); Would panic here
                    /// ```
                    #[must_use]
                    #[inline(always)]
                    $vis const fn into_a(self) -> $a_name {
                        if 0 $cmp 1 {
                            assert!(self.get() <= $a_ty::MAX as $b_ty);
                        }
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
        $vis:vis struct $base_ident:ident: $(#[feature($feature:literal)] $var_ty:ident $(as $var_ident:ident)?),+;
    } => {
        $(
            #[cfg(feature = $feature)]
            #[cfg_attr(not(feature = $feature), allow(rust_analyzer::inactive_code))]
            concat_idents::concat_idents!(struct_ident = $base_ident $(, $var_ident)? {
                /// Zero cost abstraction to add differenciate data size types.
                /// # Examples
                /// ```
                /// use anysize::*;
                /// let bits = BitsU8::new(10);
                /// assert_eq!(bits.get(), 10u8);
                ///
                /// let bytes = BytesU8::new(2);
                /// assert_eq!(bytes.get(), 2u8);
                ///
                /// let words = WordsU8::of::<u128>();
                /// assert_eq!(words.get(), 4);
                ///
                /// // Sizes are equal to the suffix
                /// assert_eq!(size_of::<BytesU8>(), size_of::<u8>());
                /// assert_eq!(size_of::<BitsU32>(), size_of::<u32>());
                /// assert_eq!(size_of::<WordsUSize>(), size_of::<usize>());
                ///
                /// // Types are universally interchangeable
                /// let bits = BitsU8::new(16);
                /// assert_eq!(bits.get(), 16);
                /// let bytes = bits.as_bytes();
                /// assert_eq!(bytes.get(), 2);
                /// // A.as_B asks the question: "How many B do I need to fit A"
                /// // So you need for example 2 bytes for 12 bits
                /// let words = bytes.as_words();
                /// assert_eq!(words.get(), 1);
                /// let words_to_bits = words.as_bits();
                /// assert_eq!(words_to_bits.get(), 32);
                /// assert_ne!(bits, words_to_bits);
                ///
                /// // Sizes also
                /// let bits = BitsU8::new(200);
                /// assert_eq!(bits.get(), 200);
                /// // bits + BitsU8::new(100) would panic, because 200 + 100 > u8::MAX
                /// let mut bits = bits.as_u32();
                /// assert_eq!(bits.get(), 200);
                /// bits += BitsU32::new(100);
                /// assert_eq!(bits.get(), 300);
                ///
                /// // Get
                /// assert_eq!(WordsU8::BITS, BitsU8::new(32));
                /// let bytes = BytesUSize::of::<u32>();
                /// assert_eq!(bytes.get(), size_of::<u32>());
                ///
                /// // Math
                /// assert_eq!(BytesU8::new(2) + BytesU8::new(6), BytesU8::new(8));
                /// ```
                #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
                $vis struct struct_ident {
                   pub(self) inner: $var_ty
                }

                impl Clone for struct_ident {
                    fn clone(&self) -> Self {
                        Self::new(self.inner)
                    }
                }
                impl Copy for struct_ident {}
                impl Default for struct_ident {
                    fn default() -> Self {
                        Self::ZERO
                    }
                }

                impl struct_ident {
                    concat_idents::concat_idents!(bits_ident = Bits $(, $var_ident)? {
                        /// Amount of bits, describing the size of this type
                        /// # Examples
                        /// ```
                        /// use anysize::*;
                        /// assert_eq!(BitsU8::BITS.get(), 1);
                        /// assert_eq!(BytesU32::BITS.get(), 8);
                        /// assert_eq!(WordsU32::BITS.get(), 32);
                        /// ```
                        $vis const BITS: bits_ident = bits_ident::new($bit_size);
                    });

                    $vis const ZERO: Self = Self::new(0);
                    $vis const ONE: Self = Self::new(1);
                    $vis const MAX: Self = Self::new($var_ty::MAX);

                    /// Will create a new instance of the given type
                    /// # Examples
                    /// ```
                    /// use anysize::*;
                    /// let bytes = BytesU32::new(10);
                    /// assert_eq!(bytes.get(), 10);
                    /// ```
                    #[must_use]
                    #[inline(always)]
                    $vis const fn new($words: $var_ty) -> Self {
                        Self { inner: $words }
                    }
                    /// Gets the value as integer
                    /// # Examples
                    /// ```
                    /// use anysize::*;
                    /// let bytes = BytesU32::new(10);
                    /// assert_eq!(bytes.get(), 10);
                    /// ```
                    #[must_use]
                    #[inline(always)]
                    $vis const fn get(self) -> $var_ty {
                        self.inner
                    }
                    /// Creates new instance, depending on the size of a type
                    /// # Examples
                    /// ```
                    /// use anysize::*;
                    /// assert_eq!(BitsU32::of::<u128>().get(), 128);
                    /// assert_eq!(WordsU8::of::<u64>().get(), 2);
                    /// assert_eq!(BytesUSize::of::<isize>().get(), size_of::<isize>());
                    /// ```
                    #[must_use]
                    $vis const fn of<T>() -> Self {
                        let size = if $bit_size < 8 {
                            size_of::<T>() * (8 / $bit_size)
                        } else if $bit_size == 8 {
                            size_of::<T>()
                        } else {
                            size_of::<T>().div_ceil($bit_size / 8)
                        };
                        assert!(size <= $var_ty::MAX as usize);
                        Self::new(size as $var_ty)
                    }
                    /// Returns the next power of two
                    #[must_use]
                    #[inline(always)]
                    $vis const fn next_power_of_two(self) -> Self {
                        Self::new(self.get().next_power_of_two())
                    }
                    /// Returns if the value is a power of two
                    #[must_use]
                    #[inline(always)]
                    $vis const fn is_power_of_two(self) -> bool {
                        self.get().is_power_of_two()
                    }

                    /// Const addition
                    #[must_use]
                    #[inline(always)]
                    $vis const fn add(self, other: Self) -> Self {
                        Self::new(self.inner + other.inner)
                    }
                    /// Const subtraction
                    #[must_use]
                    #[inline(always)]
                    $vis const fn sub(self, other: Self) -> Self {
                        Self::new(self.inner - other.inner)
                    }
                    /// Const multiplication with integer
                    #[must_use]
                    #[inline(always)]
                    $vis const fn mul(self, other: $var_ty) -> Self {
                        Self::new(self.inner * other)
                    }
                    /// Const division with integer
                    #[must_use]
                    #[inline(always)]
                    $vis const fn div(self, other: $var_ty) -> Self {
                        Self::new(self.inner / other)
                    }
                    /// Const ceil rounding devision with integer
                    #[must_use]
                    #[inline(always)]
                    $vis const fn div_ceil(self, other: $var_ty) -> Self {
                        Self::new(self.inner.div_ceil(other))
                    }

                    /// Const eq
                    #[must_use]
                    #[inline(always)]
                    $vis const fn eq(self, other: Self) -> bool {
                        self.inner == other.inner
                    }
                    /// Const ne
                    #[must_use]
                    #[inline(always)]
                    $vis const fn ne(self, other: Self) -> bool {
                        self.inner != other.inner
                    }
                    /// Const gt
                    #[must_use]
                    #[inline(always)]
                    $vis const fn gt(self, other: Self) -> bool {
                        self.inner > other.inner
                    }
                    /// Const ge
                    #[must_use]
                    #[inline(always)]
                    $vis const fn ge(self, other: Self) -> bool {
                        self.inner >= other.inner
                    }
                    /// Const lt
                    #[must_use]
                    #[inline(always)]
                    $vis const fn lt(self, other: Self) -> bool {
                        self.inner < other.inner
                    }
                    /// Const le
                    #[must_use]
                    #[inline(always)]
                    $vis const fn le(self, other: Self) -> bool {
                        self.inner <= other.inner
                    }

                    /// Const min
                    #[must_use]
                    #[inline(always)]
                    $vis const fn min(self, other: Self) -> Self {
                        if self.lt(other) {
                            self
                        } else {
                            other
                        }
                    }
                    /// Const max
                    #[must_use]
                    #[inline(always)]
                    $vis const fn max(self, other: Self) -> Self {
                        if self.gt(other) {
                            self
                        } else {
                            other
                        }
                    }
                    /// Const clamp
                    #[must_use]
                    #[inline(always)]
                    $vis const fn clamp(self, min: Self, max: Self) -> Self {
                        assert!(min.le(max));
                        if self.le(min) {
                            min
                        } else if self.ge(max) {
                            max
                        } else {
                            self
                        }
                    }
                }

                impl Add for struct_ident {
                    type Output = Self;

                    #[inline(always)]
                    fn add(self, rhs: Self) -> Self::Output {
                        self.add(rhs)
                    }
                }
                impl AddAssign for struct_ident {
                    #[inline(always)]
                    fn add_assign(&mut self, rhs: Self) {
                        *self = self.add(rhs);
                    }
                }
                impl Sub for struct_ident {
                    type Output = Self;

                    #[inline(always)]
                    fn sub(self, rhs: Self) -> Self::Output {
                        self.sub(rhs)
                    }
                }
                impl SubAssign for struct_ident {
                    #[inline(always)]
                    fn sub_assign(&mut self, rhs: Self) {
                        *self = self.sub(rhs);
                    }
                }
                impl Into<$var_ty> for struct_ident {
                    #[inline(always)]
                    fn into(self) -> $var_ty {
                        self.get()
                    }
                }
                impl From<$var_ty> for struct_ident {
                    #[inline(always)]
                    fn from(value:$var_ty) -> struct_ident {
                        Self::new(value)
                    }
                }

                impl fmt::Binary for struct_ident {
                    #[inline(always)]
                    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                        fmt::Binary::fmt(&self.inner, f)
                    }
                }
                impl fmt::UpperHex for struct_ident {
                    #[inline(always)]
                    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                        fmt::Binary::fmt(&self.inner, f)
                    }
                }
                impl fmt::LowerHex for struct_ident {
                    #[inline(always)]
                    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                        fmt::LowerHex::fmt(&self.inner, f)
                    }
                }
                impl fmt::Octal for struct_ident {
                    #[inline(always)]
                    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                        fmt::Octal::fmt(&self.inner, f)
                    }
                }
                impl fmt::UpperExp for struct_ident {
                    #[inline(always)]
                    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                        fmt::UpperExp::fmt(&self.inner, f)
                    }
                }
                impl fmt::LowerExp for struct_ident {
                    #[inline(always)]
                    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                        fmt::LowerExp::fmt(&self.inner, f)
                    }
                }
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
                        if unit < UNITS.len().min(u8::MAX as usize) as u8 {
                            f.write_char(UNITS[unit as usize] as char)?;
                        }
                        f.write_str(if integer == 1 && fraction == 0 {
                            &stringify!($words)[0..(stringify!($words).len() - 1)]
                        } else {
                            stringify!($words)
                        })
                    }
                }
                impl fmt::Debug for struct_ident {
                    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                        fmt::Display::fmt(&self.inner, f)?;
                        f.write_char(' ')?;
                        f.write_str(if self.inner == 1 {
                            &stringify!($words)[0..(stringify!($words).len() - 1)]
                        } else {
                            stringify!($words)
                        })
                    }
                }
            });
        )+
    };
}
macro_rules! generate {
    ($vis:vis $words:ident = $bit_size:literal as $name:ident $(, impl Into<$(#[feature($feature:literal)] $impl_ty:ident as $impl_words:ident),+$(,)?>)*) => {
        generate_base! {
            $words = $bit_size;
            $vis struct $name:
                #[feature("u8")] u8 as U8,
                #[feature("u16")] u16 as U16,
                #[feature("u32")] u32 as U32,
                #[feature("u64")] u64 as U64,
                #[feature("u128")] u128 as U128,
                #[feature("usize")] usize as USize;
        }
        generate_size_conversion_fn! {
            $vis $name as $words {
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
                    $vis $name => $impl_ty as $impl_words {
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
generate!(pub bits = 1 as Bits, impl Into<
    #[feature("bytes")] Bytes as bytes,
    #[feature("hwords")] HWords as hwords,
    #[feature("words")] Words as words,
    #[feature("dwords")] DWords as dwords,
    #[feature("qwords")] QWords as qwords,
>);

#[cfg(feature = "bytes")]
generate!(pub bytes = 8 as Bytes, impl Into<
    #[feature("bits")] Bits as bits,
    #[feature("hwords")] HWords as hwords,
    #[feature("words")] Words as words,
    #[feature("dwords")] DWords as dwords,
    #[feature("qwords")] QWords as qwords,
>);

#[cfg(feature = "hwords")]
generate!(pub hwords = 16 as HWords, impl Into<
    #[feature("bits")] Bits as bits,
    #[feature("bytes")] Bytes as bytes,
    #[feature("words")] Words as words,
    #[feature("dwords")] DWords as dwords,
    #[feature("qwords")] QWords as qwords,
>);

#[cfg(feature = "words")]
generate!(pub words = 32 as Words, impl Into<
    #[feature("bits")] Bits as bits,
    #[feature("bytes")] Bytes as bytes,
    #[feature("hwords")] HWords as hwords,
    #[feature("dwords")] DWords as dwords,
    #[feature("qwords")] QWords as qwords,
>);

#[cfg(feature = "dwords")]
generate!(pub dwords = 64 as DWords, impl Into<
    #[feature("bits")] Bits as bits,
    #[feature("bytes")] Bytes as bytes,
    #[feature("hwords")] HWords as hwords,
    #[feature("words")] Words as words,
    #[feature("qwords")] QWords as qwords,
>);

#[cfg(feature = "qwords")]
generate!(pub qwords = 128 as QWords, impl Into<
    #[feature("bits")] Bits as bits,
    #[feature("bytes")] Bytes as bytes,
    #[feature("hwords")] HWords as hwords,
    #[feature("words")] Words as words,
    #[feature("dwords")] DWords as dwords,
>);
