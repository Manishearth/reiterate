use elsa::FrozenVec;

use stable_deref_trait::StableDeref;
use std::cell::{Cell, RefCell};
use std::ops::Deref;

/// An adaptor around an iterator that can produce multiple iterators
/// sharing an underlying cache.
///
/// The underlying iterator must produce heap-allocated StableDeref values,
/// e.g. Box or String. If you have an iterator that produces Copy values,
/// use `CopyReiterator` instead.
///
/// ```rust
/// use reiterate::Reiterate;
///
/// let x = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()];
/// let reiterate = Reiterate::new(x);
/// for i in &reiterate {
///     println!("{}", i);    
/// }
///
/// for i in &reiterate {
///     // will reuse cached values
///     println!("{}", i);    
/// }
/// ```
///
/// In case your values are not heap-allocated, use `.map(Box::new)`:
///
/// ```rust
/// use reiterate::Reiterate;
/// let x = vec![1, 2, 3, 4];
///
/// let reiterate = Reiterate::new(x.into_iter().map(Box::new));
/// for i in &reiterate {
///     println!("{}", i);
/// }
/// ```
pub struct Reiterate<I>
where
    I: Iterator,
    I::Item: StableDeref,
{
    iter: RefCell<I>,
    curr: Cell<usize>,
    cache: FrozenVec<I::Item>,
}

impl<I> Reiterate<I>
where
    I: Iterator,
    I::Item: StableDeref,
{
    pub fn new<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = I::Item, IntoIter = I>,
    {
        Reiterate {
            iter: RefCell::new(iter.into_iter()),
            cache: FrozenVec::new(),
            curr: Cell::new(0),
        }
    }
}

impl<'a, I> IntoIterator for &'a Reiterate<I>
where
    I: Iterator,
    I::Item: StableDeref,
{
    type IntoIter = Reiterator<'a, I>;
    type Item = &'a <I::Item as Deref>::Target;

    fn into_iter(self) -> Self::IntoIter {
        Reiterator {
            iterable: self,
            curr: 0,
        }
    }
}

/// An individual iterator, produced by calling `.into_iter()` on an `&Reiterate` instance
pub struct Reiterator<'a, I>
where
    I: Iterator,
    I::Item: StableDeref,
{
    iterable: &'a Reiterate<I>,
    curr: usize,
}

impl<'a, I> Iterator for Reiterator<'a, I>
where
    I: Iterator,
    I::Item: StableDeref + Sized,
{
    type Item = &'a <I::Item as Deref>::Target;

    fn next(&mut self) -> Option<Self::Item> {
        let itercurr = self.iterable.curr.get();
        if self.curr == itercurr {
            self.iterable.curr.set(itercurr + 1);
            let val = self.iterable.iter.borrow_mut().next();
            self.curr += 1;
            if let Some(val) = val {
                self.iterable.cache.push(val)
            }
            return self.iterable.cache.get(self.curr - 1);
        } else if self.curr > itercurr {
            return None;
        } else {
            self.curr += 1;
            return self.iterable.cache.get(self.curr - 1);
        }
    }
}

/// An adaptor around an iterator over Copy items that can produce multiple iterators
/// sharing an underlying cache.
///
/// The underlying iterator must produce Copy values. If your values aren't Copy, please
/// use `Reiterator` instead.
///
/// ```rust
/// use reiterate::CopyReiterate;
///
/// let x = vec!["a", "b", "c", "d"];
/// let reiterate = CopyReiterate::new(x);
/// for i in &reiterate {
///     println!("{}", i);    
/// }
///
/// for i in &reiterate {
///     // will reuse cached values
///     println!("{}", i);
/// }
/// ```
pub struct CopyReiterate<I>
where
    I: Iterator,
    I::Item: Copy,
{
    inner: RefCell<CopyReiterateInner<I>>,
}

struct CopyReiterateInner<I>
where
    I: Iterator,
    I::Item: Copy,
{
    iter: I,
    curr: usize,
    cache: Vec<I::Item>,
}

impl<I> CopyReiterate<I>
where
    I: Iterator,
    I::Item: Copy,
{
    pub fn new<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = I::Item, IntoIter = I>,
    {
        CopyReiterate {
            inner: RefCell::new(CopyReiterateInner {
                iter: iter.into_iter(),
                cache: Vec::new(),
                curr: 0,
            }),
        }
    }
}

impl<'a, I> IntoIterator for &'a CopyReiterate<I>
where
    I: Iterator,
    I::Item: Copy,
{
    type IntoIter = CopyReiterator<'a, I>;
    type Item = I::Item;

    fn into_iter(self) -> Self::IntoIter {
        CopyReiterator {
            iterable: self,
            curr: 0,
        }
    }
}

/// An individual iterator, produced by calling `.into_iter()` on an `&CopyReiterate` instance
pub struct CopyReiterator<'a, I>
where
    I: Iterator,
    I::Item: Copy,
{
    iterable: &'a CopyReiterate<I>,
    curr: usize,
}

impl<'a, I> Iterator for CopyReiterator<'a, I>
where
    I: Iterator,
    I::Item: Copy + Sized,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let mut iterable = self.iterable.inner.borrow_mut();
        if self.curr == iterable.curr {
            iterable.curr += 1;
            let val = iterable.iter.next();
            self.curr += 1;
            if let Some(val) = val {
                iterable.cache.push(val)
            }
            return iterable.cache.get(self.curr - 1).cloned();
        } else if self.curr > iterable.curr {
            return None;
        } else {
            self.curr += 1;
            return iterable.cache.get(self.curr - 1).cloned();
        }
    }
}
