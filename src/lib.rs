//! Check yourself before you wreck yourself.
//!
//! This is a small utility library inspired by the ideas of "Parse, don't validate"<sup>[1]</sup>
//! and its follow-up, "Names are not type safety"<sup>[2]</sup>. Its goal is to extend the idea to
//! checking invariants more generally.
//!
//! [1]: https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/
//! [2]: https://lexi-lambda.github.io/blog/2020/11/01/names-are-not-type-safety/
//!
//! # Motivating example
//!
//! The motivating use-case for this crate was validating signed messages. Consider a `Signed`
//! struct like the following:
//!
//! ```
//! # struct PublicKey;
//! # struct Signature;
//! struct Signed {
//!     payload: Vec<u8>,
//!     public_key: PublicKey,
//!     signature: Signature,
//! }
//! ```
//!
//! The struct contains a payload, a public key, and a signature. Let's give the struct a `validate`
//! method that we could use to check for validity:
//!
//! ```
//! # struct PublicKey;
//! # type Error = ();
//! # impl PublicKey {
//! #     fn verify(&self, payload: &[u8], signature: &Signature) -> Result<(), Error> {
//! #         todo!()
//! #     }
//! # }
//! # struct Signature;
//! # struct Signed {
//! #     payload: Vec<u8>,
//! #     public_key: PublicKey,
//! #     signature: Signature,
//! # }
//! impl Signed {
//!     fn validate(&self) -> Result<(), Error> {
//!         self.public_key.verify(&self.payload, &self.signature)
//!     }
//! }
//! ```
//!
//! Now when we find a `Signed` we're able to verify it. Of course, whenever we see a `Signed` in
//! our code, it may not immediately be clear whether it has been checked yet. In particular, if
//! `Signed` appears in another struct, or as a signature to some method, has it already been
//! checked? Should we check it anyway?
//!
//! It's possible to manage this with disciplined use of documentation and convention, making it
//! clear where signatures should be validated and relying on that being the case later in the call
//! stack. However discipline is not always a reliable tool, particularly in an evolving codebase
//! with multiple contributors. Perhaps we can do something better?
//!
//! ## Parse, don't validate
//!
//! This is where the ideas from "Parse, don't validate" come in. Specifically, rather than
//! validating a `Signed` instance, we could 'parse' it into something else, such as
//! `CheckedSigned`:
//!
//! ```
//! # struct PublicKey;
//! # type Error = ();
//! # impl PublicKey {
//! #     fn verify(&self, payload: &[u8], signature: &Signature) -> Result<(), Error> {
//! #         todo!()
//! #     }
//! # }
//! # struct Signature;
//! # struct Signed {
//! #     payload: Vec<u8>,
//! #     public_key: PublicKey,
//! #     signature: Signature,
//! # }
//! /// A [`Signed`] that has been checked and confirmed to be valid.
//! struct CheckedSigned(Signed);
//!
//! impl CheckedSigned {
//!     fn try_from(signed: Signed) -> Result<Self, Error> {
//!         signed.public_key.verify(&signed.payload, &signed.signature)?;
//!         Ok(Self(signed))
//!     }
//! }
//! ```
//!
//! By having `CheckedSigned` in its own module, and keeping its field private, we can guarantee
//! that the only way to construct one is via the `try_from` method, which performs the check. This
//! means that structs and functions can use `CheckedSigned` and safely assume that the signature is
//! valid.
//!
//! ```
//! # struct CheckedSigned;
//! fn process_message(message: CheckedSigned) {
//!     /* ... */
//! }
//!
//! // Or
//!
//! struct ProcessMessage {
//!     message: CheckedSigned,
//! }
//! ```
//!
//! It's immediately clear in both cases that `message` has already been checked, and is known to be
//! valid.
//!
//! So far so good, but since `CheckedSigned`'s field is private, we've lost direct access to the
//! inner value. Rust makes it easy to recover some functionality here by implementing
//! [`Deref`](core::ops::Deref) for `CheckedSigned`:
//!
//! ```
//! # struct Signed;
//! # struct CheckedSigned(Signed);
//! impl core::ops::Deref for CheckedSigned {
//!     type Target = Signed;
//!     fn deref(&self) -> &Self::Target {
//!         &self.0
//!     }
//! }
//! ```
//!
//! This allows `Signed` methods with the `&self` receiver to be called directly on `CheckedSigned`
//! instances.
//!
//! ## So what about this library...?
//!
//! Creating a `Checked*` newtype for every type that needs checked would be a lot of boilerplate,
//! and there are many ways to skin this cat, so to speak. `check_mate` exists to offer a consistent
//! pattern, with minimal boilerplate.
//!
//! # How to use
//!
//! Let's start again from our original `Signed` struct above:
//!
//! ```
//! # struct PublicKey;
//! # struct Signature;
//! struct Signed {
//!     payload: Vec<u8>,
//!     public_key: PublicKey,
//!     signature: Signature,
//! }
//! ```
//!
//! We can use `check_mate` to achieve the same guarantees as `CheckedSigned` by implementing
//! [`Check`]:
//!
//! ```
//! # struct PublicKey;
//! # impl PublicKey {
//! #     fn verify(&self, payload: &[u8], signature: &Signature) -> Result<(), Error> {
//! #         todo!()
//! #     }
//! # }
//! # struct Signature;
//! # type Error = ();
//! # struct Signed {
//! #     payload: Vec<u8>,
//! #     public_key: PublicKey,
//! #     signature: Signature,
//! # }
//! impl check_mate::Check for Signed {
//!     type Ok = Self;
//!     type Err = Error;
//!
//!     fn check(self) -> Result<Self::Ok, Self::Err> {
//!         self.public_key.verify(&self.payload, &self.signature)?;
//!         Ok(self)
//!     }
//! }
//! ```
//!
//! Now we can obtain a [`Checked`]`<Signed>` using [`try_from`](Checked::try_from):
//!
//! ```no_run
//! # struct PublicKey;
//! # impl PublicKey {
//! #     fn verify(&self, payload: &[u8], signature: &Signature) -> Result<(), Error> {
//! #         todo!()
//! #     }
//! # }
//! # struct Signature;
//! # type Error = ();
//! # struct Signed {
//! #     payload: Vec<u8>,
//! #     public_key: PublicKey,
//! #     signature: Signature,
//! # }
//! # impl check_mate::Check for Signed {
//! #     type Ok = Self;
//! #     type Err = Error;
//! #
//! #     fn check(self) -> Result<Self::Ok, Self::Err> {
//! #         self.public_key.verify(&self.payload, &self.signature)?;
//! #         Ok(self)
//! #     }
//! # }
//! # let signed: Signed = todo!();
//! let _ = check_mate::Checked::try_from(signed);
//! ```
//!
//! `Checked<T>` implements `Deref<Target = T>`, and can be converted back to the inner value with
//! [`into_inner`](Checked::into_inner).
//!
//! With the `serde` feature enabled, `Checked<T>` will also implement `Serialize` if
//! `T: Serialize`, and `Deserialize` if `T: Deserialize` **and** there's a `Check<Ok = T>` impl to
//! use for the check (unconstrained type parameter limitations prevent a blanket `Deserialize` impl
//! for any `U: Check<Ok = T>` – it must be `T` itself).
//!
//! # When (not) to use this
//!
//! It's hoped that `check_mate` will be useful for getting started with this 'parsing' style of
//! maintaining invariants, and for internal APIs where churn is likely, so reducing the amount of
//! code involved is desired.
//!
//! However, `Checked<T>` can't be as ergonomic or featureful as a custom checked type could. For
//! example, if it's known that some fields don't affect validity they could be made public, or
//! methods that don't affect validity could take `&mut self`. Neither of these are possible with
//! `Checked<T>` since the inner value is only exposed immutably.
//!
//! It may also be unsuitable when you want a great deal of customisation over how validation is
//! performed. This *could* be achieved either by including configuration in the type that
//! implements `Check`, or otherwise by implementing `Check` on wrappers that can tailor the
//! behaviour, but it would likely be a bit clunky to use.
//!
//! Finally, as discussed in "Names are not type safety", it's always preferable to design types
//! that simply cannot represent invalid states, though it may not always be possible.
//!
//! # What's next?
//!
//! I want to try and use this to get a sense of whether or not it's actually useful, and what the
//! pain points are. Some things I could imagine adding:
//!
//! - Implement additional common traits (`AsRef<T>`, `Borrow<T>`).
//! - Implement additional common indirection methods (`as_deref`, `cloned`).

#![warn(clippy::pedantic)]
#![cfg_attr(not(test), no_std)]

/// A checked value.
///
/// The wrapped value is guaranteed to be valid with respect to its implementation of [`Check`].
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Checked<T>(T);

impl<T> Checked<T> {
    /// Check a value.
    ///
    /// # Errors
    ///
    /// This will return the error from [`Check::check`] verbatim if the check fails.
    pub fn try_from<U: Check<Ok = T>>(value: U) -> Result<Self, U::Err> {
        value.check().map(Checked)
    }
}

impl<T: Check<Err = core::convert::Infallible>> Checked<T> {
    /// Construct a checked value.
    ///
    /// Rather than generating a value known to be valid, then having to check it, this can be used
    /// to immediately construct a valid value, so long as the [`Check`] implementation doesn't
    /// fail.
    pub fn from(value: T) -> Checked<T::Ok> {
        value.check().map(Checked).expect("infallible")
    }
}

impl<T> Checked<T> {
    /// Retrieve the inner value, dropping the 'proof' that it was checked.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> core::ops::Deref for Checked<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for Checked<T>
where
    T: serde::Deserialize<'de> + Check<Ok = T>,
    T::Err: core::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let value = T::deserialize(deserializer)?;
        Self::try_from(value).map_err(D::Error::custom)
    }
}

/// Checked values.
pub trait Check {
    /// The value returned when the check passes.
    ///
    /// This will often be `Self`, but it's specified as an associated type to allow for information
    /// to be lost from the checked value.
    type Ok;

    /// The error returned when the check fails.
    type Err;

    /// Check `self`.
    ///
    /// # Errors
    ///
    /// If `self` is valid this should return `Ok(Self::Ok)`, and otherwise `Err(Self::Err)`.
    fn check(self) -> Result<Self::Ok, Self::Err>;
}

#[cfg(test)]
mod tests {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
    struct LessThan10(usize);

    impl Check for LessThan10 {
        type Ok = Self;
        type Err = &'static str;

        fn check(self) -> Result<Self::Ok, Self::Err> {
            if self.0 < 10 {
                Ok(self)
            } else {
                Err("too big")
            }
        }
    }

    struct GenLessThan10;

    impl Check for GenLessThan10 {
        type Ok = LessThan10;
        type Err = core::convert::Infallible;

        fn check(self) -> Result<Self::Ok, Self::Err> {
            Ok(LessThan10(3))
        }
    }

    use super::{Check, Checked};

    #[test]
    fn try_from() {
        assert_eq!(
            Checked::try_from(LessThan10(9)).as_deref(),
            Ok(&LessThan10(9))
        );

        assert_eq!(
            Checked::try_from(LessThan10(11)).as_deref(),
            Err(&"too big")
        );
    }

    #[test]
    fn from() {
        assert_eq!(&*Checked::from(GenLessThan10), &LessThan10(3));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize() {
        assert_eq!(
            serde_json::from_str::<Checked<LessThan10>>("3")
                .ok()
                .as_deref(),
            Some(&LessThan10(3))
        );

        assert_eq!(
            serde_json::from_str::<Checked<LessThan10>>("10")
                .err()
                .map(|error| error.to_string()),
            Some("too big".to_string())
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize() {
        assert_eq!(
            serde_json::to_string(&Checked::from(GenLessThan10)).unwrap(),
            serde_json::to_string(&LessThan10(3)).unwrap()
        );
    }
}
