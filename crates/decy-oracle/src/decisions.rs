//! C→Rust specific decision categories

use serde::{Deserialize, Serialize};
use std::fmt;

/// C→Rust specific decision categories
///
/// These categories extend entrenar's generic decision types with
/// C-specific constructs that commonly cause ownership/lifetime errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CDecisionCategory {
    // Ownership inference (most critical)
    /// Pointer ownership: *T → Box<T> vs &T vs &mut T
    PointerOwnership,
    /// Array ownership: T[] → Vec<T> vs &[T] vs Box<[T]>
    ArrayOwnership,
    /// String ownership: char* → String vs &str vs CString
    StringOwnership,

    // Lifetime inference
    /// When to elide vs explicit 'a
    LifetimeElision,
    /// Struct field lifetime annotations
    StructLifetime,
    /// Return reference lifetime binding
    ReturnLifetime,

    // Unsafe minimization
    /// When unsafe is truly necessary
    UnsafeBlock,
    /// *const T → &T safety
    RawPointerCast,
    /// NULL → Option<T> wrapping
    NullCheck,

    // Type mapping
    /// int → i32 vs i64 vs isize
    IntegerPromotion,
    /// C enum → Rust enum
    EnumMapping,
    /// C union → Rust enum or unsafe union
    UnionMapping,

    // Concurrency
    /// pthread_mutex_t → Mutex<T>
    MutexWrapping,
    /// _Atomic → std::sync::atomic
    AtomicMapping,
}

impl fmt::Display for CDecisionCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PointerOwnership => write!(f, "pointer_ownership"),
            Self::ArrayOwnership => write!(f, "array_ownership"),
            Self::StringOwnership => write!(f, "string_ownership"),
            Self::LifetimeElision => write!(f, "lifetime_elision"),
            Self::StructLifetime => write!(f, "struct_lifetime"),
            Self::ReturnLifetime => write!(f, "return_lifetime"),
            Self::UnsafeBlock => write!(f, "unsafe_block"),
            Self::RawPointerCast => write!(f, "raw_pointer_cast"),
            Self::NullCheck => write!(f, "null_check"),
            Self::IntegerPromotion => write!(f, "integer_promotion"),
            Self::EnumMapping => write!(f, "enum_mapping"),
            Self::UnionMapping => write!(f, "union_mapping"),
            Self::MutexWrapping => write!(f, "mutex_wrapping"),
            Self::AtomicMapping => write!(f, "atomic_mapping"),
        }
    }
}

impl CDecisionCategory {
    /// Returns true if this category relates to ownership inference
    pub fn is_ownership(&self) -> bool {
        matches!(
            self,
            Self::PointerOwnership | Self::ArrayOwnership | Self::StringOwnership
        )
    }

    /// Returns true if this category relates to lifetime inference
    pub fn is_lifetime(&self) -> bool {
        matches!(
            self,
            Self::LifetimeElision | Self::StructLifetime | Self::ReturnLifetime
        )
    }

    /// Returns true if this category relates to unsafe code
    pub fn is_unsafe(&self) -> bool {
        matches!(
            self,
            Self::UnsafeBlock | Self::RawPointerCast | Self::NullCheck
        )
    }

    /// Error codes commonly associated with this decision category
    pub fn associated_errors(&self) -> &'static [&'static str] {
        match self {
            Self::PointerOwnership => &["E0382", "E0499", "E0506"],
            Self::ArrayOwnership => &["E0382", "E0499", "E0506"],
            Self::StringOwnership => &["E0382", "E0308"],
            Self::LifetimeElision => &["E0597", "E0515"],
            Self::StructLifetime => &["E0597", "E0515"],
            Self::ReturnLifetime => &["E0515", "E0597"],
            Self::UnsafeBlock => &["E0133"],
            Self::RawPointerCast => &["E0133", "E0606"],
            Self::NullCheck => &["E0308"],
            Self::IntegerPromotion => &["E0308"],
            Self::EnumMapping => &["E0308"],
            Self::UnionMapping => &["E0133", "E0308"],
            Self::MutexWrapping => &["E0382", "E0499"],
            Self::AtomicMapping => &["E0308"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_display() {
        assert_eq!(CDecisionCategory::PointerOwnership.to_string(), "pointer_ownership");
        assert_eq!(CDecisionCategory::UnsafeBlock.to_string(), "unsafe_block");
    }

    #[test]
    fn test_category_display_all() {
        // Ownership
        assert_eq!(CDecisionCategory::ArrayOwnership.to_string(), "array_ownership");
        assert_eq!(CDecisionCategory::StringOwnership.to_string(), "string_ownership");

        // Lifetime
        assert_eq!(CDecisionCategory::LifetimeElision.to_string(), "lifetime_elision");
        assert_eq!(CDecisionCategory::StructLifetime.to_string(), "struct_lifetime");
        assert_eq!(CDecisionCategory::ReturnLifetime.to_string(), "return_lifetime");

        // Unsafe
        assert_eq!(CDecisionCategory::RawPointerCast.to_string(), "raw_pointer_cast");
        assert_eq!(CDecisionCategory::NullCheck.to_string(), "null_check");

        // Type mapping
        assert_eq!(CDecisionCategory::IntegerPromotion.to_string(), "integer_promotion");
        assert_eq!(CDecisionCategory::EnumMapping.to_string(), "enum_mapping");
        assert_eq!(CDecisionCategory::UnionMapping.to_string(), "union_mapping");

        // Concurrency
        assert_eq!(CDecisionCategory::MutexWrapping.to_string(), "mutex_wrapping");
        assert_eq!(CDecisionCategory::AtomicMapping.to_string(), "atomic_mapping");
    }

    #[test]
    fn test_category_classification() {
        assert!(CDecisionCategory::PointerOwnership.is_ownership());
        assert!(!CDecisionCategory::PointerOwnership.is_lifetime());

        assert!(CDecisionCategory::LifetimeElision.is_lifetime());
        assert!(!CDecisionCategory::LifetimeElision.is_ownership());

        assert!(CDecisionCategory::UnsafeBlock.is_unsafe());
    }

    #[test]
    fn test_all_ownership_categories() {
        assert!(CDecisionCategory::PointerOwnership.is_ownership());
        assert!(CDecisionCategory::ArrayOwnership.is_ownership());
        assert!(CDecisionCategory::StringOwnership.is_ownership());

        // Non-ownership
        assert!(!CDecisionCategory::LifetimeElision.is_ownership());
        assert!(!CDecisionCategory::UnsafeBlock.is_ownership());
        assert!(!CDecisionCategory::IntegerPromotion.is_ownership());
    }

    #[test]
    fn test_all_lifetime_categories() {
        assert!(CDecisionCategory::LifetimeElision.is_lifetime());
        assert!(CDecisionCategory::StructLifetime.is_lifetime());
        assert!(CDecisionCategory::ReturnLifetime.is_lifetime());

        // Non-lifetime
        assert!(!CDecisionCategory::PointerOwnership.is_lifetime());
        assert!(!CDecisionCategory::UnsafeBlock.is_lifetime());
    }

    #[test]
    fn test_all_unsafe_categories() {
        assert!(CDecisionCategory::UnsafeBlock.is_unsafe());
        assert!(CDecisionCategory::RawPointerCast.is_unsafe());
        assert!(CDecisionCategory::NullCheck.is_unsafe());

        // Non-unsafe
        assert!(!CDecisionCategory::PointerOwnership.is_unsafe());
        assert!(!CDecisionCategory::IntegerPromotion.is_unsafe());
    }

    #[test]
    fn test_associated_errors() {
        let errors = CDecisionCategory::PointerOwnership.associated_errors();
        assert!(errors.contains(&"E0382"));
        assert!(errors.contains(&"E0499"));
    }

    #[test]
    fn test_associated_errors_all_categories() {
        // Verify all categories have associated errors
        let all_categories = [
            CDecisionCategory::PointerOwnership,
            CDecisionCategory::ArrayOwnership,
            CDecisionCategory::StringOwnership,
            CDecisionCategory::LifetimeElision,
            CDecisionCategory::StructLifetime,
            CDecisionCategory::ReturnLifetime,
            CDecisionCategory::UnsafeBlock,
            CDecisionCategory::RawPointerCast,
            CDecisionCategory::NullCheck,
            CDecisionCategory::IntegerPromotion,
            CDecisionCategory::EnumMapping,
            CDecisionCategory::UnionMapping,
            CDecisionCategory::MutexWrapping,
            CDecisionCategory::AtomicMapping,
        ];

        for cat in all_categories {
            let errors = cat.associated_errors();
            assert!(!errors.is_empty(), "Category {:?} has no associated errors", cat);
        }
    }

    #[test]
    fn test_transferable_error_codes() {
        // E0382 (borrow of moved value) should appear in ownership categories
        assert!(CDecisionCategory::PointerOwnership.associated_errors().contains(&"E0382"));
        assert!(CDecisionCategory::ArrayOwnership.associated_errors().contains(&"E0382"));
        assert!(CDecisionCategory::MutexWrapping.associated_errors().contains(&"E0382"));

        // E0597 (borrowed value does not live long enough) in lifetime categories
        assert!(CDecisionCategory::LifetimeElision.associated_errors().contains(&"E0597"));
        assert!(CDecisionCategory::StructLifetime.associated_errors().contains(&"E0597"));
        assert!(CDecisionCategory::ReturnLifetime.associated_errors().contains(&"E0597"));

        // E0133 (unsafe block required) in unsafe categories
        assert!(CDecisionCategory::UnsafeBlock.associated_errors().contains(&"E0133"));
        assert!(CDecisionCategory::RawPointerCast.associated_errors().contains(&"E0133"));
        assert!(CDecisionCategory::UnionMapping.associated_errors().contains(&"E0133"));
    }
}
