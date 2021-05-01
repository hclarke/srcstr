use core::hash::Hash;
use core::hash::Hasher;
use core::ops::Deref;
use core::ops::Index;
use core::ops::{Range, RangeFrom, RangeTo, RangeFull, RangeInclusive, RangeToInclusive };
use std::rc::Rc;
use core::fmt;

#[derive(Clone)]
pub struct SrcStr {
    rc: Rc<String>,
    ptr: *const str, // either points into the owner, or 'static
}

impl PartialEq for SrcStr {
    fn eq(&self, rhs: &Self) -> bool {
        self.rc.as_ptr() == rhs.rc.as_ptr() && self.ptr == rhs.ptr
    }
}
impl Eq for SrcStr {}

impl Hash for SrcStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rc.as_ptr().hash(state);
        self.ptr.hash(state);
    }
}

impl fmt::Debug for SrcStr {
	fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
		// TODO: put context in here? (full line. maybe only with #? debug format)
		f.write_str(self.deref())
	}
}

impl From<Rc<String>> for SrcStr {
    fn from(rc: Rc<String>) -> Self {
        let ptr = (&**rc) as *const str;
        Self { rc, ptr }
    }
}

impl From<String> for SrcStr {
    fn from(string: String) -> Self {
        Rc::new(string).into()
    }
}

impl<'a> From<&'a str> for SrcStr {
	fn from(string: &'a str) -> Self {
		string.to_string().into()
	}
}

impl Deref for SrcStr {
    type Target = str;
    fn deref(&self) -> &str {
        unsafe { &*self.ptr }
    }
}

impl From<SrcStr> for String {
    fn from(ss: SrcStr) -> String {
        (*ss).to_string()
    }
}

impl Index<Range<usize>> for SrcStr {
    type Output = str;
    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.deref()[index]
    }
}

impl Index<RangeFrom<usize>> for SrcStr {
    type Output = str;
    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        &self.deref()[index]
    }
}

impl Index<RangeTo<usize>> for SrcStr {
    type Output = str;
    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        &self.deref()[index]
    }
}

impl Index<RangeInclusive<usize>> for SrcStr {
    type Output = str;
    fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
        &self.deref()[index]
    }
}

impl Index<RangeToInclusive<usize>> for SrcStr {
    type Output = str;
    fn index(&self, index: RangeToInclusive<usize>) -> &Self::Output {
        &self.deref()[index]
    }
}

impl Index<RangeFull> for SrcStr {
    type Output = str;
    fn index(&self, index: RangeFull) -> &Self::Output {
        &self.deref()[index]
    }
}

impl SrcStr {
    pub fn src(&self) -> &Rc<String> {
        &self.rc
    }

    pub fn try_run<T, E, F>(&mut self, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut Self) -> Result<T, E>,
    {
        let ptr = self.ptr;
        let result = f(self);
        if result.is_err() {
            self.ptr = ptr;
        }
        result
    }

    pub fn edit<T, F>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut &str) -> T,
    {
        let mut s = &**self;
        let result = f(&mut s);
        self.ptr = s as *const str;

        result
    }

    pub fn try_edit<T, E, F>(&mut self, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut &str) -> Result<T, E>,
    {
        self.try_run(|this| this.edit(f))
    }

    pub fn range(&self) -> Option<Range<usize>> {
        let outer = &self.rc[..];
        let inner = &**self;


        let start = outer.as_bytes() as *const [u8] as *const u8 as usize;
        let len = outer.len();
        let end = start + len;

        let ptr = self.ptr as *const [u8] as *const u8 as usize;

        if ptr < start || ptr >= end {
            return None;
        }

        let ptr_start = ptr-start;
        let ptr_end = ptr_start + inner.len();

        Some(ptr_start..ptr_end)

    }

    pub fn sub(&self, index: Range<usize>) -> SrcStr {
    	let mut s = self.clone();
    	s.edit(move |s| *s = &s[index]);
    	s
    }

    pub fn src_sub(&self, index: Range<usize>) -> SrcStr {
        let mut s = self.clone();
        let ptr = &s.src()[index] as *const str;
        s.ptr = ptr;
        s
    }
}





#[cfg(test)]
mod tests {
	use super::*;

    #[test]
    fn index() {
        let s: SrcStr = "A pair of powerful spectacles has sometimes sufficed to cure a person in love.".into();
        assert_eq!("powerful", &s[10..18]);
        assert_eq!(s, s);
    }

    #[test]
    fn hash() {
    	let a: SrcStr = "Even the most courageous among us only has the courage for that which he really knows.".into();

    	let b = a.sub(14..21);
    	let c = a.sub(47..54);
    	
    	assert_eq!("courage", &b[..]);
    	assert_eq!(&b[..], &c[..]);
    	assert_ne!(b, c);
    }

    #[test]
    fn range() {
        let mut a: SrcStr = "Thoughts are the shadows of our feelings - always darker, emptier and simpler.".into();

        a.edit(|s| *s = &s[15..18]);

        assert_eq!(a.range(), Some(15..18)); 

        a.edit(|s| *s = "nothing");

        assert_eq!(a.range(), None);
    }
}
