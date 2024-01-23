use serde::{Serialize, Deserialize};

/// The location of a string. Used by [`crate::rules::conditions::Condition::UrlPartContains`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum StringLocation {
    /// Checks if an instance of the needle exists anywhere in the haystack.
    /// # Examples
    /// ```
    /// # use url_cleaner::types::StringLocation;
    /// assert!(StringLocation::Anywhere.satisfied_by("abcdef", "cde").is_ok_and(|x| x==true ));
    /// assert!(StringLocation::Anywhere.satisfied_by("abcdef", "efg").is_ok_and(|x| x==false));
    /// ````
    #[default]
    Anywhere,
    /// Checks if an instance of the needle exists at the start of the haystack.
    /// # Examples
    /// ```
    /// # use url_cleaner::types::StringLocation;
    /// assert!(StringLocation::Start.satisfied_by("abcdef", "abc").is_ok_and(|x| x==true ));
    /// assert!(StringLocation::Start.satisfied_by("abcdef", "bcd").is_ok_and(|x| x==false));
    /// ````
    Start,
    /// Checks if an instance of the needle exists at the end of the haystack.
    /// # Examples
    /// ```
    /// # use url_cleaner::types::StringLocation;
    /// assert!(StringLocation::End.satisfied_by("abcdef", "def").is_ok_and(|x| x==true ));
    /// assert!(StringLocation::End.satisfied_by("abcdef", "cde").is_ok_and(|x| x==false));
    /// ````
    End,
    /// Checks if an instance of the needle starts and ends at the specified range in the haystack.
    /// # Examples
    /// ```
    /// # use url_cleaner::types::StringLocation;
    /// assert!(StringLocation::RangeIs{start: 0, end: 3}.satisfied_by("abcdef", "abc"   ).is_ok_and(|x| x==true ));
    /// assert!(StringLocation::RangeIs{start: 1, end: 4}.satisfied_by("abcdef", "bcd"   ).is_ok_and(|x| x==true ));
    /// assert!(StringLocation::RangeIs{start: 0, end: 6}.satisfied_by("abcdef", "abcdef").is_ok_and(|x| x==true ));
    /// assert!(StringLocation::RangeIs{start: 5, end: 6}.satisfied_by("abcdef", "f"     ).is_ok_and(|x| x==true ));
    /// assert!(StringLocation::RangeIs{start: 6, end: 7}.satisfied_by("abcdef", "f"     ).is_err());
    /// assert!(StringLocation::RangeIs{start: 7, end: 8}.satisfied_by("abcdef", "f"     ).is_err());
    /// ````
    RangeIs {
        /// The start of the range to check.
        start: usize,
        /// The end of the range to check.
        end: usize
    },
    /// Checks if an instance of the needle starts at the specified point in the haystack.
    /// # Examples
    /// ```
    /// # use url_cleaner::types::StringLocation;
    /// assert!(StringLocation::StartsAt(0).satisfied_by("abcdef", "abc").is_ok_and(|x| x==true ));
    /// assert!(StringLocation::StartsAt(1).satisfied_by("abcdef", "bcd").is_ok_and(|x| x==true ));
    /// assert!(StringLocation::StartsAt(5).satisfied_by("abcdef", "f"  ).is_ok_and(|x| x==true ));
    /// assert!(StringLocation::StartsAt(0).satisfied_by("abcdef", "bcd").is_ok_and(|x| x==false));
    /// assert!(StringLocation::StartsAt(1).satisfied_by("abcdef", "cde").is_ok_and(|x| x==false));
    /// assert!(StringLocation::StartsAt(5).satisfied_by("abcdef", "def").is_ok_and(|x| x==false));
    /// ````
    StartsAt(usize),
    /// Checks if an instance of the needle ends at the specified point in the haystack.
    /// # Examples
    /// ```
    /// # use url_cleaner::types::StringLocation;
    /// assert!(StringLocation::EndsAt(3).satisfied_by("abcdef", "abc").is_ok_and(|x| x==true ));
    /// assert!(StringLocation::EndsAt(4).satisfied_by("abcdef", "bcd").is_ok_and(|x| x==true ));
    /// assert!(StringLocation::EndsAt(6).satisfied_by("abcdef", "def").is_ok_and(|x| x==true ));
    /// assert!(StringLocation::EndsAt(6).satisfied_by("abcdef", "f"  ).is_ok_and(|x| x==true ));
    /// assert!(StringLocation::EndsAt(3).satisfied_by("abcdef", "bcd").is_ok_and(|x| x==false));
    /// assert!(StringLocation::EndsAt(4).satisfied_by("abcdef", "cde").is_ok_and(|x| x==false));
    /// ````
    EndsAt(usize),
    /// Checks if an instance of the needle exists within the specified range of the haystack.
    /// # Examples
    /// ```
    /// # use url_cleaner::types::StringLocation;
    /// assert!(StringLocation::RangeHas{start: 0, end: 1}.satisfied_by("abcdef", "a"   ).is_ok_and(|x| x==true ));
    /// assert!(StringLocation::RangeHas{start: 0, end: 2}.satisfied_by("abcdef", "a"   ).is_ok_and(|x| x==true ));
    /// assert!(StringLocation::RangeHas{start: 0, end: 6}.satisfied_by("abcdef", "bcde").is_ok_and(|x| x==true ));
    /// assert!(StringLocation::RangeHas{start: 1, end: 6}.satisfied_by("abcdef", "a"   ).is_ok_and(|x| x==false));
    /// assert!(StringLocation::RangeHas{start: 0, end: 7}.satisfied_by("abcdef", ""    ).is_err());
    /// ````
    RangeHas {
        /// The start of the range to check.
        start: usize,
        /// The end of the range to check.
        end: usize
    },
    /// Checks if an instance of the needle exists after the specified point in the haystack.
    /// # Examples
    /// ```
    /// # use url_cleaner::types::StringLocation;
    /// assert!(StringLocation::After(0).satisfied_by("abcdef", "abcdef").is_ok_and(|x| x==true ));
    /// assert!(StringLocation::After(1).satisfied_by("abcdef", "bcdef" ).is_ok_and(|x| x==true ));
    /// assert!(StringLocation::After(1).satisfied_by("abcdef", "1"     ).is_ok_and(|x| x==false));
    /// assert!(StringLocation::After(6).satisfied_by("abcdef", "f"     ).is_ok_and(|x| x==false));
    /// assert!(StringLocation::After(7).satisfied_by("abcdef", ""      ).is_err());
    /// ````
    After(usize),
    /// Checks if an instance of the needle exists before the specified point in the haystack.
    /// # Examples
    /// ```
    /// # use url_cleaner::types::StringLocation;
    /// assert!(StringLocation::Before(0).satisfied_by("abcdef", ""   ).is_ok_and(|x| x==true ));
    /// assert!(StringLocation::Before(1).satisfied_by("abcdef", "a"  ).is_ok_and(|x| x==true ));
    /// assert!(StringLocation::Before(6).satisfied_by("abcdef", "a"  ).is_ok_and(|x| x==true ));
    /// assert!(StringLocation::Before(4).satisfied_by("abcdef", "def").is_ok_and(|x| x==false ));
    /// assert!(StringLocation::Before(7).satisfied_by("abcdef", "a"  ).is_err());
    /// ````
    Before(usize)
}

impl StringLocation {
    /// Checks if `needle` exists in `haystack` according to `self`'s rules.
    /// # Errors
    /// If only part of the haystack is searched and that part either is out of bounds or splits a UTF-8 codepoint, returns the error [`suoer::StringError::InvalidSlice`].
    pub fn satisfied_by(&self, haystack: &str, needle: &str) -> Result<bool, super::StringError> {
        Ok(match self {
            Self::Anywhere             => haystack.contains   (needle),
            Self::Start                => haystack.starts_with(needle),
            Self::End                  => haystack.ends_with  (needle),

            Self::RangeIs {start, end} => haystack.get(*start..*end).ok_or(super::StringError::InvalidSlice)?==needle,
            Self::StartsAt(start     ) => haystack.get(*start..    ).ok_or(super::StringError::InvalidSlice)?.starts_with(needle),
            Self::EndsAt  (       end) => haystack.get(      ..*end).ok_or(super::StringError::InvalidSlice)?.ends_with(needle),

            Self::RangeHas{start, end} => haystack.get(*start..*end).ok_or(super::StringError::InvalidSlice)?.contains(needle),
            Self::After   (start     ) => haystack.get(*start..    ).ok_or(super::StringError::InvalidSlice)?.contains(needle),
            Self::Before  (       end) => haystack.get(      ..*end).ok_or(super::StringError::InvalidSlice)?.contains(needle)
        })
    }
}
